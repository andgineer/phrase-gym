import type { Browser, BrowserContext, BrowserContextOptions } from '@playwright/test'

export async function silencePlayback(context: BrowserContext) {
  await context.addInitScript(() => {
    const play = HTMLMediaElement.prototype.play
    HTMLMediaElement.prototype.play = function () {
      this.volume = 0
      this.muted = true
      return play.call(this)
    }
  })
}

export async function createSilentContext(browser: Browser, options?: BrowserContextOptions) {
  const context = await browser.newContext(options)
  await silencePlayback(context)
  return context
}
