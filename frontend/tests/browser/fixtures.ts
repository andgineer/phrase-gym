import { test as base } from '@playwright/test'

export const test = base.extend<{ isolatedLibrary: void; silentPlayback: void }>({
  // Page-level fault routes cannot intercept requests owned by a service worker.
  serviceWorkers: 'block',
  silentPlayback: [
    async ({ context }, use) => {
      await context.addInitScript(() => {
        const play = HTMLMediaElement.prototype.play
        const volumes: number[] = []
        Object.assign(window, { __testPlaybackVolumes: volumes })
        HTMLMediaElement.prototype.play = function () {
          // Keep native decoding/playback events while silencing synthetic test tones.
          this.volume = 0
          // Audible WebKit playback becomes the host's Now Playing session, so a headset
          // disconnect or media key could pause it mid-test; muted media is never eligible.
          this.muted = true
          volumes.push(this.volume)
          return play.call(this)
        }
      })
      await use()
    },
    { auto: true },
  ],
  isolatedLibrary: [
    async ({ request }, use) => {
      const response = await request.post('/_test/reset')
      if (!response.ok()) throw new Error(`Fixture reset failed (${response.status()})`)
      await use()
    },
    { auto: true },
  ],
})
