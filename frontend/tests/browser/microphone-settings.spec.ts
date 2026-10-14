import { expect } from '@playwright/test'
import { test } from './capture-fixtures'
import { retainedAudio, recordingPeak, syntheticCapture } from './synthetic-capture'

test('recording uses native microphone defaults and successive takes contain an audio signal', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const acquire = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices)
    const requests: MediaStreamConstraints[] = []
    Object.assign(window, { __microphoneConstraints: requests })
    navigator.mediaDevices.getUserMedia = async (constraints) => {
      requests.push(constraints ?? {})
      const stream = await acquire(constraints)
      // Model encoded silence, which still has bytes, with processing forced off.
      const gain = (window as unknown as { __captureGain: GainNode }).__captureGain
      gain.gain.value =
        typeof constraints?.audio === 'object' && constraints.audio.echoCancellation === false
          ? 0
          : 1
      return stream
    }
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const session = await (
    await request.get(`/api/v1/sessions/${page.url().split('/').pop()}`)
  ).json()
  const recordings = `/api/v1/phrases/${session.attempts[0].target_id}/recordings`
  for (let index = 0; index < 3; index++) {
    await page.getByRole('button', { name: 'Record', exact: true }).click()
    await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
    await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
    await expect
      .poll(async () => (await (await request.get(recordings)).json()).length)
      .toBe(index + 1)
    const rows = await (await request.get(recordings)).json()
    expect(await recordingPeak(page, rows[index].id)).toBeGreaterThan(0.1)
    await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  }
  expect(
    await page.evaluate(
      () =>
        (window as unknown as { __microphoneConstraints: MediaStreamConstraints[] })
          .__microphoneConstraints,
    ),
  ).toEqual([{ audio: true }, { audio: true }, { audio: true }])
})
