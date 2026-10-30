import type { Page } from '@playwright/test'

export async function installFakeMedia(page: Page) {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, 'mediaDevices', {
      configurable: true,
      value: {
        getUserMedia: async () => ({
          getTracks: () => [],
          getAudioTracks: () => [
            {
              readyState: 'live',
              muted: false,
              stop() {},
              getSettings: () => ({ sampleRate: 48000, channelCount: 1 }),
              onended: null,
            },
          ],
        }),
      },
    })
    class SilentRecorder {
      static isTypeSupported() {
        return true
      }
      state = 'inactive'
      mimeType = 'audio/webm;codecs=opus'
      ondataavailable: ((event: { data: Blob }) => void) | null = null
      onstop: (() => void) | null = null
      onerror: (() => void) | null = null
      constructor(_stream: unknown, _options: unknown) {}
      start() {
        this.state = 'recording'
      }
      stop() {
        this.state = 'inactive'
        this.ondataavailable?.({
          data: new Blob([new Uint8Array([1, 2, 3, 4])], { type: this.mimeType }),
        })
        queueMicrotask(() => this.onstop?.())
      }
    }
    Object.defineProperty(window, 'MediaRecorder', { configurable: true, value: SilentRecorder })
    HTMLMediaElement.prototype.play = async function () {
      const host = window as unknown as {
        __playedMedia?: HTMLMediaElement[]
        __playedUrls?: string[]
      }
      ;(host.__playedMedia ??= []).push(this)
      // Playback cleanup clears src, so keep what each play started with.
      ;(host.__playedUrls ??= []).push(this.src)
    }
    HTMLMediaElement.prototype.pause = function () {
      this.dispatchEvent(new Event('pause'))
    }
  })
}
