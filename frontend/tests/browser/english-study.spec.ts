import { expect } from '@playwright/test'
import { test } from './preview-server'
import { fillLesson } from './lesson-intake-helpers'
import { englishLesson } from './english-lesson'
import { prepareWithNativeFakeCoach } from './native-coach'
import { retainedAudio, syntheticCapture } from './synthetic-capture'
import { expectComparison } from './native-playback'

test('automatic raw intake through native Coach reaches normal Start, ungraded Next and retained replay', async ({
  page,
  request,
  preview,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await syntheticCapture(page)
  const recordings: { attempt_id?: string }[] = []
  page.on('request', (r) => {
    if (r.method() === 'POST' && r.url().endsWith('/api/v1/recordings'))
      recordings.push(r.postDataJSON())
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
  await fillLesson(page)
  await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
  const publication = await prepareWithNativeFakeCoach(preview.url)
  await expect(
    page.getByText('Published. Open the lesson to see its supported practice options.', {
      exact: true,
    }),
  ).toBeVisible({ timeout: 15000 })
  await expect(page.getByRole('button', { name: 'Start session', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText("Let's get started.")
  await expect(page.locator('.practice-card')).toContainText(
    'Everyone has arrived. Open the meeting.',
  )
  await expect(page.locator('.practice-card')).toContainText('Begin a shared activity.')
  await expect(page.getByRole('button', { name: /Recalled$/ })).toHaveCount(0)
  await page.getByRole('button', { name: /Reference$/ }).click()
  await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Stop playback', exact: true }).click()
  await page.evaluate(() => {
    ;(window as unknown as { __nativePlaybacks: unknown[] }).__nativePlaybacks.length = 0
  })
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await expect(page.getByRole('button', { name: 'Next', exact: true })).toBeDisabled()
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expectComparison(page, 1)
  const takes = `/api/v1/phrases/${publication.target_ids[0]}/recordings`
  await expect.poll(async () => (await (await request.get(takes)).json()).length).toBe(1)
  const [take] = await (await request.get(takes)).json()
  const bytes = await (await request.get(`/api/v1/recordings/${take.id}`)).body()
  const state = await (await request.get('/api/v1/sessions/current')).json()
  expect(recordings).toHaveLength(1)
  expect(recordings[0].attempt_id).toBe(state.attempts[0].id)
  await page.getByRole('button', { name: 'Next', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
  const finished = await (await request.get(`/api/v1/sessions/${state.id}`)).json()
  expect(finished.attempts[0].rating).toBeNull()
  expect(finished.attempts[0].original.supported).toBe(true)
  expect(finished.attempts[0].eligibility_reason).toBe('ungraded_study')
  const receipts = await (await request.get(`/api/v1/sessions/${state.id}/events`)).json()
  expect(receipts.every((r: { schedule_changed: boolean }) => !r.schedule_changed)).toBe(true)
  expect(receipts.at(-1)).toMatchObject({
    eligibility_reason: 'ungraded_study',
    schedule_changed: false,
  })
  for (const restart of [false, true]) {
    if (restart) await preview.restart()
    await page.reload()
    await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
    expect(await (await request.get(`/api/v1/recordings/${take.id}`)).body()).toEqual(bytes)
  }
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: /My recording$/ })).toBeEnabled()
  await page.getByRole('button', { name: /My recording$/ }).click()
  await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
})

for (const phase of ['permission', 'recording', 'comparison'] as const) {
  test(`opening intake during ${phase} preserves initiating study without sending or grading`, async ({
    page,
    request,
  }) => {
    await englishLesson(request)
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    await syntheticCapture(page, phase === 'permission')
    let deliveries = 0
    page.on('request', (r) => {
      if (r.method() === 'POST' && r.url().endsWith('/api/v1/inbox/lessons')) deliveries++
    })
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Start session', exact: true }).click()
    await page.getByRole('button', { name: 'Record', exact: true }).click()
    if (phase === 'permission') {
      await expect(
        page.getByRole('button', { name: 'Cancel microphone request', exact: true }),
      ).toBeVisible()
      await expect
        .poll(() =>
          page.evaluate(
            () => (window as unknown as { __captureRequests: number }).__captureRequests,
          ),
        )
        .toBe(1)
    } else {
      await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
      if (phase === 'comparison') {
        await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
        await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
      }
    }
    const before = await (await request.get('/api/v1/sessions/current')).json()
    await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
    await page.getByRole('button', { name: 'Close', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
    const after = await (await request.get('/api/v1/sessions/current')).json()
    expect(after.id).toBe(before.id)
    expect(after.cursor).toBe(0)
    expect(after.attempts[0].id).toBe(before.attempts[0].id)
    expect(after.attempts[0].original).toEqual(before.attempts[0].original)
    expect(after.attempts[0].rating).toBeNull()
    expect(deliveries).toBe(0)
    if (phase !== 'permission') {
      const path = `/api/v1/phrases/${after.manifest.targets[0].phrase.id}/recordings`
      await expect.poll(async () => (await (await request.get(path)).json()).length).toBe(1)
    }
  })
}
