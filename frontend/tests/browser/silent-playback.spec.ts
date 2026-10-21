import { expect } from '@playwright/test'
import { test } from './fixtures'
import { createSilentContext } from './silent-playback'
import { tone } from './seed-library'

for (const separate of [false, true]) {
  test(`native media stays muted with real decoding, clocks and events, separate context=${separate}`, async ({
    context,
    browser,
  }) => {
    const selected = separate ? await createSilentContext(browser) : context
    try {
      // A broken mute guard must fail this regression without producing an audible tone.
      const silence = Buffer.from(tone(), 'base64')
      silence.fill(0, 44)
      const page = await selected.newPage()
      await page.goto('about:blank')
      const result = await page.evaluate(async (audio) => {
        const media = new Audio(`data:audio/wav;base64,${audio}`)
        // Try normal application settings; the test boundary must mute before native playback.
        media.volume = 1
        media.muted = false
        const ended = new Promise<void>((resolve, reject) => {
          media.addEventListener('ended', () => resolve(), { once: true })
          media.addEventListener('error', () => reject(new Error('Native decode failed')), {
            once: true,
          })
        })
        await media.play()
        const state = { volume: media.volume, muted: media.muted }
        await ended
        const states = (
          window as unknown as { __testPlaybackStates: { volume: number; muted: boolean }[] }
        ).__testPlaybackStates
        return { ...state, time: media.currentTime, states }
      }, silence.toString('base64'))
      expect(result.volume).toBe(0)
      expect(result.muted).toBe(true)
      expect(result.time).toBeGreaterThan(0)
      expect(result.states).toEqual([{ volume: 0, muted: true }])
    } finally {
      if (separate) await selected.close()
    }
  })
}
