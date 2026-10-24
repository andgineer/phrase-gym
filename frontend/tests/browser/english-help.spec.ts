import { expect } from '@playwright/test'
import { test } from './preview-server'
import { englishLesson } from './english-lesson'
import { syntheticCapture, retainedAudio } from './synthetic-capture'
import { expectComparison } from './native-playback'

const short = 'Begin a shared activity.'
const translation = 'Давайте начнём.'

for (const reveal of ['explain', 'answer', 'reference', 'comparison'] as const) {
  test(`the short explanation follows the ${reveal} reveal and native details only follow Show & explain`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    await englishLesson(request)
    if (reveal === 'comparison') await syntheticCapture(page)
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Start session', exact: true }).click()
    await expect(page.locator('.answer-help')).toHaveText(short)
    await expect(page.locator('.answer-help')).toHaveAttribute('lang', 'en')
    await expect(page.getByText(translation, { exact: true })).toHaveCount(0)
    await expect(page.locator('.lesson-links')).toHaveText('Open lesson →')
    if (reveal === 'explain') {
      await page.getByRole('button', { name: /Show & explain/ }).click()
      await expect(page.locator('.phrase-details')).toContainText(translation)
    }
    await page.getByRole('button', { name: 'Next', exact: true }).click()
    await page.getByRole('button', { name: 'Library', exact: true }).click()
    await page.getByRole('button', { name: 'Start session', exact: true }).click()
    await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
    await expect(page.getByText(short, { exact: true })).toHaveCount(0)
    await expect(page.getByText(translation, { exact: true })).toHaveCount(0)
    await expect(page.locator('.lesson-links')).toHaveCount(0)
    await expect
      .poll(
        async () =>
          (await (await request.get('/api/v1/sessions/current')).json())?.attempts.length ?? 0,
      )
      .toBe(1)
    const before = await (await request.get('/api/v1/sessions/current')).json()
    expect(before.attempts[0].original).toBeNull()
    if (reveal === 'comparison') {
      await page.getByRole('button', { name: 'Record', exact: true }).click()
      await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
      await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
      await expectComparison(page, 1)
    } else {
      await page
        .getByRole('button', {
          name:
            reveal === 'explain'
              ? /Show & explain/
              : reveal === 'answer'
                ? 'Show answer'
                : /Reference$/,
        })
        .click()
    }
    await expect(page.getByTestId('practice-phrase')).toHaveText("Let's get started.")
    await expect(page.locator('.answer-help')).toHaveText(short)
    await expect(page.locator('.lesson-links')).toHaveText('Open lesson →')
    const details = page.locator('.phrase-details')
    if (reveal === 'explain') {
      await expect(details).toHaveAttribute('lang', 'ru')
      await expect(details).toContainText(translation)
      await expect(details).toContainText('Нейтрально; так говорит ведущий')
      await expect(details.getByRole('heading', { name: 'Перевод' })).toBeVisible()
    } else {
      await expect(details).toHaveCount(0)
      await expect(page.getByText(translation, { exact: true })).toHaveCount(0)
    }
    await expect(page.getByRole('button', { name: /Recalled$/ })).toBeEnabled()
    await page.reload()
    await expect(page.locator('.answer-help')).toHaveText(short)
    await expect(page.locator('.phrase-details')).toHaveCount(reveal === 'explain' ? 1 : 0)
    await page.getByRole('button', { name: /Recalled$/ }).click()
    await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
    const after = await (await request.get(`/api/v1/sessions/${before.id}`)).json()
    const supported = reveal === 'explain' || reveal === 'reference'
    expect(after.attempts[0].original.supported).toBe(supported)
    const reason = supported ? 'supported_learning_check' : 'eligible'
    expect(after.attempts[0].eligibility_reason).toBe(reason)
    const receipts = await (await request.get(`/api/v1/sessions/${before.id}/events`)).json()
    expect(receipts.at(-1).eligibility_reason).toBe(reason)
  })
}

test('a card without native details offers no Show & explain and still shows its short help', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await englishLesson(request, false)
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.locator('.answer-help')).toHaveText(short)
  await expect(page.getByRole('button', { name: /Show & explain/ })).toHaveCount(0)
})
