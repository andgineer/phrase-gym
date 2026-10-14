import { expect } from '@playwright/test'
import { test } from './capture-fixtures'
import { expectComparison } from './native-playback'
import { recordingPeak, syntheticCapture } from './synthetic-capture'

test('a longer take finishes both native comparison playbacks before recording again', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const NativeRecorder = window.MediaRecorder
    let chunks = 0
    Object.defineProperty(window, '__encodedChunks', { get: () => chunks })
    window.MediaRecorder = class extends NativeRecorder {
      constructor(stream: MediaStream, options?: MediaRecorderOptions) {
        super(stream, options)
        this.addEventListener('dataavailable', (event) => {
          if (event.data.size) chunks++
        })
      }
    }
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect
    .poll(
      () => page.evaluate(() => (window as unknown as { __encodedChunks: number }).__encodedChunks),
      { timeout: 10000 },
    )
    .toBeGreaterThanOrEqual(6)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  const playback = await expectComparison(page, 1)
  expect(playback[0]!.time).toBeGreaterThan(5)
  const session = await (
    await request.get(`/api/v1/sessions/${page.url().split('/').pop()}`)
  ).json()
  const rows = await (
    await request.get(`/api/v1/phrases/${session.attempts[0].target_id}/recordings`)
  ).json()
  expect(rows).toHaveLength(1)
  expect(rows[0].complete).toBe(true)
  expect(await recordingPeak(page, rows[0].id)).toBeGreaterThan(0.1)
})
