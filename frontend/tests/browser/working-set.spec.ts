import { expect, type Page } from '@playwright/test'
import { readFile } from 'node:fs/promises'
import { test } from './preview-server'
import { prepareWithNativeFakeCoach } from './native-coach'

// A batch's lesson is named after its items, so a card must show no sibling target's answer.
async function expectNoOtherAnswers(page: Page, targets: any[], previous: string) {
  const heading = page.getByTestId('practice-phrase')
  if (previous) await expect(heading).not.toHaveText(previous)
  const shownText = (await heading.innerText()).trim()
  const shown = targets.find((target) => target.phrase.content.text === shownText)
  expect(shown, shownText).toBeTruthy()
  const card = await page.locator('.practice-card').innerText()
  for (const target of targets.filter((target) => target.phrase.id !== shown.phrase.id))
    for (const answer of [target.phrase.content.text, target.phrase.english?.task?.word])
      if (answer) expect(card, `${answer} leaked onto another card`).not.toContain(answer)
  return shownText
}

test('ordinary five-item intake publishes fixed, frame and contextual word study then independent recall with durable matching references', async ({
  page,
  request,
  preview,
}) => {
  const fixture = JSON.parse(await readFile('../fixtures/coach/working-set.json', 'utf8'))
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Add phrase', exact: true }).click()
  await page.getByRole('textbox', { name: 'Exact English phrase' }).fill(fixture.input)
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
  const publication = await prepareWithNativeFakeCoach(preview.url)
  expect(publication.target_ids).toHaveLength(5)
  const summary = await (await request.get('/api/v1/coach/summary')).json()
  expect(summary).toEqual({ lessons: 1, targets: 5, ready: 5, study_only: 0, awaiting_audio: 0 })
  await expect(
    page.getByText('Published. Open the lesson to see its supported practice options.', {
      exact: true,
    }),
  ).toBeVisible({ timeout: 15000 })
  await page.getByRole('button', { name: 'Close', exact: true }).click()
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toBeVisible()
  let session = await (await request.get('/api/v1/sessions/current')).json()
  expect(session.manifest.targets).toHaveLength(5)
  const word = session.manifest.targets.find(
    (item: any) => item.phrase.content.kind === 'word_sense',
  )
  const frame = session.manifest.targets.find((item: any) => item.phrase.content.kind === 'frame')
  expect(word.phrase.english.task.contexts).toHaveLength(3)
  expect(frame.phrase.content.slots.item).toEqual(['the call', 'the meeting'])
  const firstId = session.id
  let shown = ''
  for (let index = 0; index < 5; index++) {
    await expect(page.getByTestId('practice-phrase')).toBeVisible()
    if (session.manifest.targets[index].phrase.content.kind === 'word_sense') {
      await expect(page.locator('.word-sense-cue')).toHaveText('Make it clear.')
      await expect(page.locator('.practice-prompt')).not.toContainText('|')
    }
    await expect(page.getByRole('button', { name: /Recalled$/ })).toHaveCount(0)
    await expect(page.locator('.lesson-links')).toHaveText('Open lesson →')
    shown = await expectNoOtherAnswers(page, session.manifest.targets, shown)
    await page.getByRole('button', { name: 'Next', exact: true }).click()
  }
  await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
  session = await (await request.get(`/api/v1/sessions/${firstId}`)).json()
  expect(
    session.attempts.every(
      (attempt: any) => attempt.rating === null && attempt.eligibility_reason === 'ungraded_study',
    ),
  ).toBe(true)
  await preview.restart()
  await page.reload()
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeVisible()
  session = await (await request.get('/api/v1/sessions/current')).json()
  shown = ''
  for (let index = 0; index < 5; index++) {
    await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
    await expect(page.locator('.target-task')).toHaveCount(0)
    const item = session.manifest.targets[index]
    if (item.phrase.content.kind === 'word_sense') {
      await expect(page.locator('.word-sense-cue')).toHaveText('Make it clear.')
      await expect(page.locator('.practice-prompt')).not.toContainText('|')
      const context = item.phrase.english.task.contexts.find(
        (context: any) => context.situation_id === item.cue_id,
      )
      expect(item.phrase.content.text).toBe(context.answer)
      expect(item.phrase.content.reference_audio_id).toBe(context.reference_audio_id)
    }
    await page.getByRole('button', { name: 'Show answer', exact: true }).click()
    await expect(page.getByTestId('practice-phrase')).toHaveText(item.phrase.content.text)
    await expect(page.locator('.lesson-links')).toHaveText('Open lesson →')
    shown = await expectNoOtherAnswers(page, session.manifest.targets, shown)
    await page.getByRole('button', { name: /Recalled$/ }).click()
  }
  await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
  const receipts = await (await request.get(`/api/v1/sessions/${session.id}/events`)).json()
  expect(receipts.filter((receipt: any) => receipt.schedule_changed)).toHaveLength(5)
  const readiness = await (await request.get('/api/v1/sessions/readiness?language=en')).json()
  expect(readiness).toMatchObject({
    practice_ready_count: 5,
    due_count: 0,
    new_count: 0,
    reference_only_count: 0,
  })
  expect(Date.parse(readiness.next_due_at)).toBeGreaterThan(Date.now())
})
