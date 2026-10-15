import { expect, test, type BrowserContext, type Page } from '@playwright/test'

type Playback = { source: string; ended: boolean; failed: boolean; time: number }

// Headless Chromium on the macOS CI runner has played media at under half of real time.
const COMPARISON_BUDGET = 45000

export async function observeNativePlayback(context: BrowserContext) {
  await context.addInitScript(() => {
    const play = HTMLMediaElement.prototype.play
    const playbacks: Playback[] = []
    const listeners = new WeakMap<HTMLMediaElement, AbortController>()
    Object.assign(window, { __nativePlaybacks: playbacks })
    HTMLMediaElement.prototype.play = function () {
      listeners.get(this)?.abort()
      const controller = new AbortController()
      listeners.set(this, controller)
      const playback = { source: this.src, ended: false, failed: false, time: 0 }
      playbacks.push(playback)
      this.addEventListener(
        'ended',
        () => {
          playback.time = this.currentTime
          playback.ended = true
          controller.abort()
        },
        { capture: true, once: true, signal: controller.signal },
      )
      this.addEventListener('error', () => (playback.failed = true), {
        capture: true,
        once: true,
        signal: controller.signal,
      })
      return play.call(this)
    }
  })
}

export async function expectComparison(page: Page, count: number) {
  // Native decoding and two real playback clocks need their own bounded budget.
  test.info().setTimeout(test.info().timeout + COMPARISON_BUDGET)
  await expect
    .poll(
      () =>
        page.evaluate(
          () => (window as unknown as { __nativePlaybacks: Playback[] }).__nativePlaybacks,
        ),
      { timeout: COMPARISON_BUDGET },
    )
    .toMatchObject(
      Array.from({ length: count }, () => [
        { source: expect.stringMatching(/^blob:/), ended: true, failed: false },
        { source: expect.stringContaining('/api/v1/media/'), ended: true, failed: false },
      ]).flat(),
    )
  const playbacks = await page.evaluate(
    () => (window as unknown as { __nativePlaybacks: Playback[] }).__nativePlaybacks,
  )
  expect(playbacks.every((playback) => playback.time > 0)).toBe(true)
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  return playbacks
}
