import { expect, test } from './fixtures'

const cards = Array.from({ length: 5 }, (_, index) => ({
  item_id: `meeting-${index + 1}`,
  answer: [
    "Let's get started.",
    'Could you walk me through that?',
    "I'll get back to you tomorrow.",
    'That works for me.',
    'Let me think about it.',
  ][index],
  explanation: 'A natural phrase for an ordinary conversation.',
  cues: [{ situation: `Use the useful expression in situation ${index + 1}.` }],
  accepted_variants: index === 0 ? ["Let's begin."] : [],
}))

const phrasePackage = {
  format: 'phrase-gym.package',
  version: 1,
  package_id: 'browser-five-phrases-v1',
  source_id: 'browser-course-unit-1',
  source_label: 'Browser course · Unit 1',
  source_link: 'https://example.test/course/1',
  learning_language: 'en',
  card_language: 'en',
  help_language: 'en',
  cards,
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    Object.assign(window, {
      __nativeMediaRecorder: window.MediaRecorder,
      __nativeMediaPlay: HTMLMediaElement.prototype.play,
    })
    Object.defineProperty(navigator, 'maxTouchPoints', { value: 0 })
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
    if (!navigator.locks)
      Object.defineProperty(navigator, 'locks', {
        value: {
          request: async (_name: string, options: unknown, callback?: Function) => {
            const run = typeof options === 'function' ? options : callback
            return run?.({ name: _name })
          },
        },
      })
    if (new URLSearchParams(location.search).has('take-transfer-race')) {
      const locks = navigator.locks
      const nativeRequest = locks.request.bind(locks)
      const host = window as unknown as {
        __releaseTakeRace?: () => void
        __takeTransferReached?: boolean
      }
      Object.defineProperty(locks, 'request', {
        configurable: true,
        value: async (...args: unknown[]) => {
          if (String(args[0]).startsWith('phrase-gym.take:') && !host.__takeTransferReached) {
            host.__takeTransferReached = true
            await new Promise<void>((release) => {
              host.__releaseTakeRace = release
            })
          }
          return Reflect.apply(nativeRequest, locks, args)
        },
      })
    }
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
      const host = window as unknown as { __playedMedia?: HTMLMediaElement[] }
      ;(host.__playedMedia ??= []).push(this)
    }
    HTMLMediaElement.prototype.pause = function () {
      this.dispatchEvent(new Event('pause'))
    }
  })
})

test('native recorder keeps variable encoder output and muted playback clocks', async ({
  page,
}) => {
  const nativePackage = {
    ...phrasePackage,
    package_id: 'native-recorder-v1',
    source_id: 'native-recorder-source',
    cards: [
      {
        ...cards[0],
        item_id: 'native-recorder-card',
        answer: 'This recording uses the browser encoder.',
      },
    ],
  }
  await page.goto('/')
  await page.evaluate(() => {
    const host = window as unknown as {
      __nativeMediaRecorder: typeof MediaRecorder
      __nativeMediaPlay: typeof HTMLMediaElement.prototype.play
      __captureAudio?: { context: AudioContext; oscillator: OscillatorNode }
      __playedMedia?: HTMLMediaElement[]
    }
    Object.defineProperty(window, 'MediaRecorder', {
      configurable: true,
      value: host.__nativeMediaRecorder,
    })
    Object.defineProperty(navigator, 'mediaDevices', {
      configurable: true,
      value: {
        getUserMedia: async () => {
          const context = new AudioContext()
          const destination = context.createMediaStreamDestination()
          const oscillator = context.createOscillator()
          oscillator.frequency.value = 220
          oscillator.connect(destination)
          oscillator.start()
          host.__captureAudio = { context, oscillator }
          return destination.stream
        },
      },
    })
    const nativePlay = host.__nativeMediaPlay
    HTMLMediaElement.prototype.play = function () {
      this.muted = true
      this.volume = 0
      ;(host.__playedMedia ??= []).push(this)
      return nativePlay.call(this)
    }
  })
  await page.getByText('Add material').click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(nativePackage))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByText('Added 1 phrases.')).toBeVisible()
  await page.getByRole('button', { name: 'Record a take' }).click()
  await page.waitForTimeout(1100)
  await page.getByRole('button', { name: 'Stop recording' }).click()
  await expect(page.locator('section.study').getByText('Take 1', { exact: true })).toBeVisible()
  const saved = await page.evaluate(async () =>
    (await fetch('/api/study/current', { method: 'POST' })).json(),
  )
  expect(saved.card.recordings[0].byte_length).toBeGreaterThan(4)
  expect(saved.card.recordings[0].mime_type).toMatch(/^audio\//)
  await page.getByRole('button', { name: 'Replay' }).click()
  await expect
    .poll(() =>
      page.evaluate(() => {
        const media = (
          window as unknown as { __playedMedia?: HTMLMediaElement[] }
        ).__playedMedia?.at(-1)
        return media ? { muted: media.muted, volume: media.volume, time: media.currentTime } : null
      }),
    )
    .toMatchObject({ muted: true, volume: 0 })
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          (window as unknown as { __playedMedia?: HTMLMediaElement[] }).__playedMedia?.at(-1)
            ?.currentTime ?? 0,
      ),
    )
    .toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Next' }).click()
  await expect(page.getByRole('heading', { name: 'Today’s study is complete' })).toBeVisible()
})

test('imports, retries, studies, and retains complete takes', async ({ page }) => {
  await page.goto('/')
  await page.getByText('Add material').click()
  await page.getByLabel('Phrase package JSON').fill('{ bad json')
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByRole('alert')).toContainText('Invalid JSON')

  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(phrasePackage))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByRole('alert')).toContainText('Synthetic preparation failure')
  await page.getByRole('button', { name: 'Retry preparation' }).click()
  await expect(page.getByText('Added 5 phrases.')).toBeVisible()
  await expect(page.getByRole('heading', { name: "Let's get started." })).toBeVisible()

  let failSave = true
  let failedSaves = 0
  await page.route('**/api/recordings', async (route) => {
    if (failSave && route.request().method() === 'POST') {
      failedSaves += 1
      await route.fulfill({ status: 503, json: { detail: 'Temporary save failure' } })
      return
    }
    await route.continue()
  })
  await page.getByRole('button', { name: 'Record a take' }).click()
  await page.getByRole('button', { name: 'Stop recording' }).click()
  await expect(page.getByText('1 take(s) waiting to save.')).toBeVisible()
  await expect(page.getByRole('alert')).toContainText('Audio remains on this device')
  await page.evaluate(() => {
    const host = window as unknown as { __retrySaveSettled?: boolean }
    let cleared = false
    const observer = new MutationObserver(() => {
      const alert = document.querySelector('[role="alert"]')
      const failed = alert?.textContent?.includes('Audio remains on this device') ?? false
      cleared ||= !failed
      if (cleared && failed) {
        host.__retrySaveSettled = true
        observer.disconnect()
      }
    })
    observer.observe(document.body, { childList: true, subtree: true, characterData: true })
  })
  await page.getByRole('button', { name: 'Retry save' }).click()
  await expect.poll(() => failedSaves).toBe(2)
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as { __retrySaveSettled?: boolean }).__retrySaveSettled ?? false,
      ),
    )
    .toBe(true)
  await page.evaluate(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => requestAnimationFrame(() => resolve())),
      ),
  )
  failSave = false
  await page.goto('/?take-transfer-race=1')
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          (window as unknown as { __takeTransferReached?: boolean }).__takeTransferReached ?? false,
      ),
    )
    .toBe(true)
  await page.evaluate(async () => {
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
      const request = indexedDB.open('phrase-gym.practice.v1', 1)
      request.onsuccess = () => resolve(request.result)
      request.onerror = () => reject(request.error)
    })
    await new Promise<void>((resolve, reject) => {
      const transaction = db.transaction('journal', 'readwrite')
      const store = transaction.objectStore('journal')
      const cursor = store.openCursor()
      cursor.onsuccess = () => {
        const current = cursor.result
        if (!current) return transaction.abort()
        if (!String(current.key).startsWith('take-v2:')) return current.continue()
        current.update({
          ...current.value,
          duration_ms: 600,
          complete: true,
          audio: new Uint8Array([1, 2, 3, 4, 5, 6]).buffer,
        })
      }
      transaction.oncomplete = () => resolve()
      transaction.onerror = transaction.onabort = () => reject(transaction.error)
    })
    ;(window as unknown as { __releaseTakeRace: () => void }).__releaseTakeRace()
    history.replaceState(null, '', '/')
  })
  await expect(page.locator('section.study').getByText('Take 1', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Record a take' }).click()
  await page.getByRole('button', { name: 'Stop recording' }).click()
  await expect(page.locator('section.study').getByText('Take 2', { exact: true })).toBeVisible()

  await page.reload()
  await expect(page.locator('section.study').getByText('Take 1', { exact: true })).toBeVisible()
  await expect(page.locator('section.study').getByText('Take 2', { exact: true })).toBeVisible()
  const activeStudy = page.locator('section.study')
  await expect(activeStudy.getByRole('button', { name: 'Replay', exact: true })).toHaveCount(2)
  await expect(activeStudy.getByRole('button', { name: 'Compare with reference' })).toHaveCount(2)
  const saved = await page.evaluate(async () =>
    (await fetch('/api/study/current', { method: 'POST' })).json(),
  )
  expect(saved.card.recordings).toEqual(
    expect.arrayContaining([
      expect.objectContaining({
        mime_type: 'audio/webm;codecs=opus',
        byte_length: 6,
        complete: true,
      }),
    ]),
  )
  await activeStudy.getByRole('button', { name: 'Compare with reference' }).first().click()
  const beforePause = await page.evaluate(
    () => (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.length,
  )
  await page.evaluate(() => {
    const media = (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.at(-1)!
    media.dispatchEvent(new Event('pause'))
    media.dispatchEvent(new Event('ended'))
  })
  expect(
    await page.evaluate(
      () => (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.length,
    ),
  ).toBe(beforePause)

  await page.getByText('Add material').click()
  await page.getByLabel('Choose JSON file').setInputFiles({
    name: 'phrases.json',
    mimeType: 'application/json',
    buffer: Buffer.from(JSON.stringify(phrasePackage)),
  })
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByText('Added 5 phrases.')).toBeVisible()

  for (let index = 0; index < 5; index++) {
    await expect(page.getByText('This first meeting is ungraded.')).toBeVisible()
    await page.getByRole('button', { name: 'Next' }).click()
  }
  await expect(page.getByRole('heading', { name: 'Today’s study is complete' })).toBeVisible()
  await expect(page.getByText('later recall is not scheduled yet')).toBeVisible()
  const firstHistory = page.getByRole('heading', { name: "Let's get started." }).locator('..')
  await expect(firstHistory.getByRole('button', { name: 'Replay saved take' })).toHaveCount(2)
})
