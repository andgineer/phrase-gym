import { expect } from '@playwright/test'
import { test } from './capture-fixtures'
import { expectComparison } from './native-playback'
import { emulateTouchDevice, retainedAudio, syntheticCapture } from './synthetic-capture'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('on touch devices each take releases the microphone before comparison switches to an exclusive playback session', async ({
  page,
  request,
}) => {
  await emulateTouchDevice(page)
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const acquire = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices)
    let stream: MediaStream | null = null
    navigator.mediaDevices.getUserMedia = async (constraints) => {
      stream = await acquire(constraints)
      return stream
    }
    let type = 'auto'
    let unsafeTransitions = 0
    Object.defineProperty(window, '__unsafePlaybackTransitions', { get: () => unsafeTransitions })
    Object.defineProperty(navigator, 'audioSession', {
      configurable: true,
      value: {
        get type() {
          return type
        },
        set type(value: string) {
          type = value
          // Playback-only sessions cannot keep a microphone track alive.
          if (value === 'playback') {
            if (stream?.getAudioTracks().some((track) => track.readyState === 'live'))
              unsafeTransitions++
            stream?.getTracks().forEach((track) => track.stop())
          }
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
    await expectComparison(page, index + 1)
    expect(
      await page.evaluate(
        () => (window as unknown as { __captureTrackState: string }).__captureTrackState,
      ),
    ).toBe('ended')
  }
  expect(
    await page.evaluate(
      () => (window as unknown as { __captureRequests: number }).__captureRequests,
    ),
  ).toBe(3)
  expect(
    await page.evaluate(
      () =>
        (window as unknown as { __unsafePlaybackTransitions: number }).__unsafePlaybackTransitions,
    ),
  ).toBe(0)
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
  await expectComparison(page, 1)
  const original = (await (await request.get(`/api/v1/sessions/${sessionId}`)).json()).attempts[0]
    .original
  expect(original).toMatchObject({ modality: 'recorded', supported: false })
  const failureResponse = page.waitForResponse(
    (response) =>
      response.url().endsWith(`/sessions/${sessionId}/events`) &&
      response.request().method() === 'POST' &&
      response
        .request()
        .postDataJSON()
        .events.some((event: { kind: string }) => event.kind === 'failure'),
  )
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Stop recording', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect(page.getByRole('alert').first()).toContainText('No audio was captured')
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  await expect.poll(() => retainedAudio(page)).toBe(0)
  await expect(page.getByText(/Recording pending/)).toHaveCount(0)
  const failed = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  const acknowledgement = await failureResponse
  expect(acknowledgement.ok()).toBe(true)
  const failure = acknowledgement
    .request()
    .postDataJSON()
    .events.find((event: { kind: string }) => event.kind === 'failure')
  const receipt = (await acknowledgement.json()).receipts[0]
  expect(receipt).toMatchObject({
    id: failure.id,
    attempt_id: session.attempts[0].id,
    schedule_changed: false,
  })
  expect(failed.attempts[0]).toMatchObject({
    technical_failure: false,
    capture_pending: false,
    original,
  })
  expect(await (await request.get(`/api/v1/sessions/${sessionId}/events`)).json()).toContainEqual(
    receipt,
  )
  expect(await (await request.get(recordings)).json()).toHaveLength(1)
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect.poll(async () => (await (await request.get(recordings)).json()).length).toBe(2)
  await expectComparison(page, 2)
  const rows = await (await request.get(recordings)).json()
  expect(
    rows.every(
      (take: { complete: boolean; byte_length: number }) => take.complete && take.byte_length > 0,
    ),
  ).toBe(true)
  await expect(page.getByRole('alert')).toHaveCount(0)
  const final = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  expect(final.attempts[0].original).toEqual(original)
  const rated = page.waitForResponse(
    (response) =>
      response.url().endsWith(`/sessions/${sessionId}/events`) &&
      response.request().method() === 'POST' &&
      response
        .request()
        .postDataJSON()
        .events.some((event: { kind: string }) => event.kind === 'rate'),
  )
  await page.getByRole('button', { name: 'Recalled' }).click()
  expect((await (await rated).json()).receipts[0]).toMatchObject({
    eligibility_reason: 'eligible',
    schedule_changed: true,
  })
  expect(
    await page.evaluate(
      () => (window as unknown as { __captureRequests: number }).__captureRequests,
    ),
    // Desktop keeps its first input and reacquires only after the empty take.
  ).toBe(2)
})

test('an interrupted microphone retains partial audio and the next take acquires a fresh input', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const acquire = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices)
    navigator.mediaDevices.getUserMedia = async (constraints) => {
      const stream = await acquire(constraints)
      Object.assign(window, {
        __interruptCapture: () => {
          const track = stream.getAudioTracks()[0]!
          track.stop()
          track.dispatchEvent(new Event('ended'))
        },
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
    if (index === 0)
      await page.evaluate(() =>
        (window as unknown as { __interruptCapture: () => void }).__interruptCapture(),
      )
    else await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
    await expect
      .poll(async () => (await (await request.get(recordings)).json()).length)
      .toBe(index + 1)
    if (index === 0)
      await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
    else await expectComparison(page, 1)
  }
  const rows = await (await request.get(recordings)).json()
  expect(rows.map((take: { complete: boolean }) => take.complete)).toEqual([false, true])
  expect(
    await page.evaluate(
      () => (window as unknown as { __captureRequests: number }).__captureRequests,
    ),
  ).toBe(2)
})
