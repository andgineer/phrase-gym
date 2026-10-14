import { expect } from '@playwright/test'
import { test } from './capture-fixtures'
import { retainedAudio, syntheticCapture } from './synthetic-capture'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('comparison preserves the microphone for successive takes with an exclusive audio session', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const acquire = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices)
    let stream: MediaStream | null = null
    navigator.mediaDevices.getUserMedia = async (constraints) => {
      stream = await acquire(constraints)
      return stream
    }
    let type = 'auto'
    Object.defineProperty(navigator, 'audioSession', {
      configurable: true,
      value: {
        get type() {
          return type
        },
        set type(value: string) {
          type = value
          // Playback-only sessions cannot keep a microphone track alive.
          if (value === 'playback') stream?.getTracks().forEach((track) => track.stop())
        },
      },
    })
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
    await expect(page.getByRole('button', { name: 'Stop recording', exact: true })).toBeVisible()
    await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
    await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
    await expect
      .poll(async () => (await (await request.get(recordings)).json()).length)
      .toBe(index + 1)
    await expect.poll(() => retainedAudio(page)).toBe(0)
    // Await both native learner/reference playbacks, including their ended events.
    await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
    expect(
      await page.evaluate(
        () => (window as unknown as { __captureTrackState: string }).__captureTrackState,
      ),
    ).toBe('live')
  }
  expect(
    await page.evaluate(
      () => (window as unknown as { __captureRequests: number }).__captureRequests,
    ),
  ).toBe(1)
  await page.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as { __captureTrackState: string }).__captureTrackState,
      ),
    )
    .toBe('ended')
})

test('an empty second take reports failure and a third take still saves', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const NativeRecorder = window.MediaRecorder
    let count = 0
    window.MediaRecorder = class extends NativeRecorder {
      constructor(stream: MediaStream, options?: MediaRecorderOptions) {
        super(stream, options)
        if (++count === 2)
          this.addEventListener('dataavailable', (event) => event.stopImmediatePropagation())
      }
    }
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const sessionId = page.url().split('/').pop()
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  const recordings = `/api/v1/phrases/${session.attempts[0].target_id}/recordings`
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect.poll(async () => (await (await request.get(recordings)).json()).length).toBe(1)
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Stop recording', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect(page.getByRole('alert').first()).toContainText('No audio was captured')
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  await expect.poll(() => retainedAudio(page)).toBe(0)
  await expect(page.getByText(/Recording pending/)).toHaveCount(0)
  const failed = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  expect(failed.attempts[0]).toMatchObject({ technical_failure: true, capture_pending: false })
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect.poll(async () => (await (await request.get(recordings)).json()).length).toBe(2)
  const rows = await (await request.get(recordings)).json()
  expect(
    rows.every(
      (take: { complete: boolean; byte_length: number }) => take.complete && take.byte_length > 0,
    ),
  ).toBe(true)
  await expect(page.getByRole('alert')).toHaveCount(0)
  expect(
    await page.evaluate(
      () => (window as unknown as { __captureRequests: number }).__captureRequests,
    ),
  ).toBe(2)
})

test('a retained microphone muted by the device is replaced before rerecording', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const acquire = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices)
    navigator.mediaDevices.getUserMedia = async (constraints) => {
      const stream = await acquire(constraints)
      Object.assign(window, {
        __muteCapture: () =>
          Object.defineProperty(stream.getAudioTracks()[0], 'muted', { value: true }),
      })
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
  for (let index = 0; index < 2; index++) {
    await page.getByRole('button', { name: 'Record', exact: true }).click()
    await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
    await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
    await expect
      .poll(async () => (await (await request.get(recordings)).json()).length)
      .toBe(index + 1)
    await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
    if (index === 0)
      await page.evaluate(() =>
        (window as unknown as { __muteCapture: () => void }).__muteCapture(),
      )
  }
  expect(
    await page.evaluate(
      () => (window as unknown as { __captureRequests: number }).__captureRequests,
    ),
  ).toBe(2)
})
