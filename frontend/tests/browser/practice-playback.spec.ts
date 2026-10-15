import { expect } from '@playwright/test'
import { test } from './capture-fixtures'
import { expectComparison } from './native-playback'
import { recordingPeak, retainedAudio, syntheticCapture } from './synthetic-capture'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('a deferred recorder MIME preserves audible bytes and compares through one player', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const NativeRecorder = window.MediaRecorder
    window.MediaRecorder = class extends NativeRecorder {
      started = false
      get mimeType() {
        return this.started ? super.mimeType : ''
      }
      start(timeslice?: number) {
        super.start(timeslice)
        this.started = true
      }
    }
    const NativeAudio = window.Audio
    const players: HTMLAudioElement[] = []
    const sources: string[] = []
    const sessionTypes: string[] = []
    let type = 'auto'
    Object.defineProperty(navigator, 'audioSession', {
      configurable: true,
      value: {
        get type() {
          return type
        },
        set type(value: string) {
          type = value
          sessionTypes.push(value)
        },
      },
    })
    window.Audio = function () {
      const player = new NativeAudio()
      players.push(player)
      player.addEventListener('play', () => sources.push(player.currentSrc || player.src))
      return player
    } as unknown as typeof Audio
    Object.assign(window, { __playbackProbe: { players, sources, sessionTypes } })
  })
  let submittedMime = ''
  await page.route('**/api/v1/recordings', async (route) => {
    submittedMime = route.request().postDataJSON().mime_type
    await route.continue()
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expectComparison(page, 1)
  expect(submittedMime).toMatch(/^audio\//)
  const probe = await page.evaluate(() => {
    const { players, sources, sessionTypes } = (
      window as unknown as {
        __playbackProbe: { players: HTMLAudioElement[]; sources: string[]; sessionTypes: string[] }
      }
    ).__playbackProbe
    return { playerCount: players.length, sources, sessionTypes }
  })
  expect(probe.playerCount).toBe(1)
  expect(probe.sources).toHaveLength(2)
  expect(probe.sources[0]).toMatch(/^blob:/)
  expect(probe.sources[1]).toContain('/api/v1/media/')
  expect(probe.sessionTypes).toContain('auto')
  expect(probe.sessionTypes).toContain('playback')
  const session = await (
    await request.get(`/api/v1/sessions/${page.url().split('/').pop()}`)
  ).json()
  const rows = await (
    await request.get(`/api/v1/phrases/${session.attempts[0].target_id}/recordings`)
  ).json()
  expect(rows).toHaveLength(1)
  expect(await recordingPeak(page, rows[0].id)).toBeGreaterThan(0.1)
  await page.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
  expect(
    await page.evaluate(
      () => (navigator as Navigator & { audioSession: { type: string } }).audioSession.type,
    ),
  ).toBe('auto')
})

test('a rejected take shows its validation reason and retains bytes without a connection warning', async ({
  page,
}) => {
  await syntheticCapture(page)
  await page.route('**/api/v1/recordings', (route) =>
    route.fulfill({
      status: 422,
      contentType: 'application/json',
      body: JSON.stringify({
        detail: [{ loc: ['body', 'mime_type'], msg: 'Recording MIME is required' }],
      }),
    }),
  )
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('body.mime_type: Recording MIME is required')
  await expect(page.getByTestId('connection-required')).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  expect(await retainedAudio(page)).toBe(1)
})
