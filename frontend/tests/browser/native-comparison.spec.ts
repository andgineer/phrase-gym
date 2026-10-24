import { expect } from '@playwright/test'
import { test } from './preview-server'
import { englishLesson } from './english-lesson'
import { allowPlayback, expectComparison, PLAYBACK_BUDGET } from './native-playback'
import { recordingSegments, syntheticCapture } from './synthetic-capture'

test('a longer take finishes both native comparison playbacks before recording again', async ({
  page,
  request,
  preview,
}) => {
  await englishLesson(request)
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  const captured = new Map<string, Buffer>()
  page.on('request', (request) => {
    if (request.method() === 'POST' && request.url().endsWith('/api/v1/recordings')) {
      const input = request.postDataJSON()
      captured.set(input.id, Buffer.from(input.audio_base64, 'base64'))
    }
  })
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const NativeRecorder = window.MediaRecorder
    let chunks = 0
    Object.defineProperty(window, '__encodedChunks', {
      get: () => chunks,
      set: (value: number) => {
        chunks = value
      },
    })
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
  await expect(page.getByTestId('practice-phrase')).toBeVisible()
  for (const [index, chunks] of [6, 3, 8].entries()) {
    await page.getByRole('button', { name: 'Record', exact: true }).click()
    await expect
      .poll(
        () =>
          page.evaluate(() => (window as unknown as { __encodedChunks: number }).__encodedChunks),
        { timeout: 10000 },
      )
      .toBeGreaterThanOrEqual(chunks)
    await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
    const playback = await expectComparison(page, index + 1)
    expect(playback[index * 2]!.time).toBeGreaterThan(chunks - 1)
    const session = await (
      await request.get(`/api/v1/sessions/${page.url().split('/').pop()}`)
    ).json()
    const rows = await (
      await request.get(`/api/v1/phrases/${session.attempts[0].target_id}/recordings`)
    ).json()
    expect(rows).toHaveLength(index + 1)
    const take = rows[index]
    expect(take.complete).toBe(true)
    const signal = await recordingSegments(page, take.id)
    expect(signal.duration).toBeGreaterThan(chunks - 1)
    expect(signal.peaks.every((peak) => peak > 0.1)).toBe(true)
    const bytes = await (await request.get(`/api/v1/recordings/${take.id}`)).body()
    expect(bytes.length).toBe(take.byte_length)
    expect(bytes).toEqual(captured.get(take.id))
    const middle = Math.floor(bytes.length / 2)
    const tail = await request.get(`/api/v1/recordings/${take.id}`, {
      headers: { Range: `bytes=${middle}-` },
    })
    expect(tail.status()).toBe(206)
    expect(await tail.body()).toEqual(bytes.subarray(middle))
    await page.evaluate(() => Object.assign(window, { __encodedChunks: 0 }))
  }
  const session = await (await request.get('/api/v1/sessions/current')).json()
  const rows = await (
    await request.get(`/api/v1/phrases/${session.attempts[0].target_id}/recordings`)
  ).json()
  const latest = rows.at(-1)
  for (const restart of [false, true]) {
    if (restart) await preview.restart()
    await page.reload()
    await expect(page.getByRole('button', { name: /My recording$/ })).toBeEnabled()
    for (const take of rows) {
      expect(await (await request.get(`/api/v1/recordings/${take.id}`)).body()).toEqual(
        captured.get(take.id),
      )
      expect((await recordingSegments(page, take.id)).peaks.every((peak) => peak > 0.1)).toBe(true)
    }
    allowPlayback()
    await page.getByRole('button', { name: /My recording$/ }).click()
    await expect
      .poll(
        () =>
          page.evaluate(
            () =>
              (
                window as unknown as {
                  __nativePlaybacks: { source: string; ended: boolean; time: number }[]
                }
              ).__nativePlaybacks[0],
          ),
        { timeout: PLAYBACK_BUDGET },
      )
      .toMatchObject({
        source: expect.stringContaining(`/recordings/${latest.id}`),
        ended: true,
        time: expect.any(Number),
      })
    const [saved] = await page.evaluate(
      () => (window as unknown as { __nativePlaybacks: { time: number }[] }).__nativePlaybacks,
    )
    expect(saved!.time).toBeGreaterThan(7)
    await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  }
})
