import type { Browser, BrowserContext, BrowserContextOptions } from '@playwright/test'

export async function silencePlayback(context: BrowserContext) {
  await context.addInitScript(() => {
    const host = window as unknown as { __testPlaybackVolumes?: number[] }
    if (host.__testPlaybackVolumes) return
    const play = HTMLMediaElement.prototype.play
    const volumes: number[] = []
    const states: { volume: number; muted: boolean }[] = []
    Object.assign(window, { __testPlaybackVolumes: volumes, __testPlaybackStates: states })
    HTMLMediaElement.prototype.play = function () {
      // Preserve native decoding, clocks and events while keeping output off the host device.
      this.volume = 0
      this.muted = true
      volumes.push(this.volume)
      states.push({ volume: this.volume, muted: this.muted })
      return play.call(this)
    }
  })
}

export async function createSilentContext(browser: Browser, options?: BrowserContextOptions) {
  const context = await browser.newContext(options)
  await silencePlayback(context)
  return context
}
