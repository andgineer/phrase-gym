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
      __nativeMediaPause: HTMLMediaElement.prototype.pause,
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
        configurable: true,
        value: {
          request: async (_name: string, options: unknown, callback?: Function) => {
            const run = typeof options === 'function' ? options : callback
            return run?.({ name: _name })
          },
        },
      })
    const locks = navigator.locks
    Object.defineProperty(navigator, 'locks', { configurable: true, value: locks })
    const nativeRequest = locks.request.bind(locks)
    const host = window as unknown as {
      __releaseTakeRace?: () => void
      __takeTransferReached?: boolean
      __lockCalls?: string[]
    }
    Object.defineProperty(locks, 'request', {
      configurable: true,
      value: async (...args: unknown[]) => {
        ;(host.__lockCalls ??= []).push(`${location.href} ${String(args[0])}`)
        if (
          // Read the destination URL when recovery requests the lock.
          new URLSearchParams(location.search).has('take-transfer-race') &&
          String(args[0]).startsWith('phrase-gym.take:') &&
          !host.__takeTransferReached
        ) {
          host.__takeTransferReached = true
          await new Promise<void>((release) => {
            host.__releaseTakeRace = release
          })
        }
        return Reflect.apply(nativeRequest, locks, args)
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
      __nativeMediaPause: typeof HTMLMediaElement.prototype.pause
      __captureAudio?: { context: AudioContext; oscillator: OscillatorNode }
      __playedMedia?: HTMLMediaElement[]
      __playedUrls?: string[]
      __playedBlobBytes?: Promise<number>
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
    HTMLMediaElement.prototype.pause = host.__nativeMediaPause
    HTMLMediaElement.prototype.play = function () {
      this.muted = true
      this.volume = 0
      ;(host.__playedMedia ??= []).push(this)
      ;(host.__playedUrls ??= []).push(this.src)
      if (this.src.startsWith('blob:'))
        host.__playedBlobBytes = fetch(this.src)
          .then((response) => response.arrayBuffer())
          .then((audio) => audio.byteLength)
      return nativePlay.call(this)
    }
  })
  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(nativePackage))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByText('Added 1 phrases.')).toBeVisible()
  await page.getByRole('button', { name: 'Close' }).click()
  await page.getByRole('button', { name: 'Record a take' }).click()
  await expect
    .poll(async () => ({
      recording: await page.getByRole('button', { name: 'Stop recording' }).isVisible(),
      errors: await page.getByRole('alert').allTextContents(),
    }))
    .toEqual({ recording: true, errors: [] })
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          (window as unknown as { __captureAudio?: { context: AudioContext } }).__captureAudio
            ?.context.currentTime ?? 0,
      ),
    )
    .toBeGreaterThan(1.1)
  await page.getByRole('button', { name: 'Stop recording' }).click()
  await expect(page.locator('article.study').getByText('Take 1', { exact: true })).toBeVisible()
  const saved = await page.evaluate(async () =>
    (await fetch('/api/study/current', { method: 'POST' })).json(),
  )
  expect(saved.card.recordings[0].byte_length).toBeGreaterThan(4)
  expect(saved.card.recordings[0].mime_type).toMatch(/^audio\//)
  expect(
    await page.evaluate(
      () => (window as unknown as { __playedBlobBytes: Promise<number> }).__playedBlobBytes,
    ),
  ).toBe(saved.card.recordings[0].byte_length)
  await expect
    .poll(() =>
      page.evaluate(() => (window as unknown as { __playedUrls?: string[] }).__playedUrls ?? []),
    )
    .toEqual([expect.stringMatching(/^blob:/), expect.stringContaining(saved.card.reference_url)])
  const afterAutomatic = await page.evaluate(
    () => (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.length,
  )
  await page.getByRole('button', { name: 'Compare with reference' }).click()
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.length,
      ),
    )
    .toBe(afterAutomatic + 1)
  await page.evaluate(() => {
    const learner = (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.at(
      -1,
    )
    if (learner) learner.onended = null
  })
  await expect
    .poll(
      () =>
        page.evaluate(
          () => (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.length,
        ),
      { timeout: 15_000 },
    )
    .toBe(afterAutomatic + 2)
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
  await expect(page.getByText('Add your first phrase batch')).toBeVisible()
  await expect(page.getByRole('heading', { name: 'Today’s study is complete' })).toHaveCount(0)
  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Phrase package JSON').fill('{ bad json')
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByRole('alert')).toContainText('Invalid JSON')

  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(phrasePackage))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByRole('alert')).toContainText('Synthetic preparation failure')
  await page.getByRole('button', { name: 'Retry preparation' }).click()
  await expect(page.getByText('Added 5 phrases.')).toBeVisible()
  await page.getByRole('button', { name: 'Close' }).click()
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
  await expect(
    page.getByRole('alert').filter({ hasText: 'Audio remains on this device' }),
  ).toBeVisible()
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
  await page.goto('/?take-transfer-race=1')
  await expect
    .poll(() =>
      page.evaluate(() => ({
        reached:
          (window as unknown as { __takeTransferReached?: boolean }).__takeTransferReached ?? false,
        calls: (window as unknown as { __lockCalls?: string[] }).__lockCalls,
        url: location.href,
        request: String(navigator.locks.request),
      })),
    )
    .toMatchObject({ reached: true })
  failSave = false
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
  await expect(page.locator('article.study').getByText('Take 1', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Record a take' }).click()
  await page.getByRole('button', { name: 'Stop recording' }).click()
  await expect(page.locator('article.study').getByText('Take 2', { exact: true })).toBeVisible()

  await page.reload()
  await expect(page.locator('article.study').getByText('Take 1', { exact: true })).toBeVisible()
  await expect(page.locator('article.study').getByText('Take 2', { exact: true })).toBeVisible()
  const activeStudy = page.locator('article.study')
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

  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Choose JSON file').setInputFiles({
    name: 'phrases.json',
    mimeType: 'application/json',
    buffer: Buffer.from(JSON.stringify(phrasePackage)),
  })
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByText('Added 5 phrases.')).toBeVisible()
  await page.getByRole('button', { name: 'Close' }).click()

  let releaseCompletion!: () => void
  const completionHeld = new Promise<void>((resolve) => {
    releaseCompletion = resolve
  })
  let completions = 0
  await page.route('**/api/attempts/*/complete', async (route) => {
    completions++
    if (completions === 1) await completionHeld
    await route.continue()
  })
  for (const [index, card] of cards.entries()) {
    await expect(
      page.locator('article.study').getByRole('heading', { name: card.answer }),
    ).toBeVisible()
    await expect(page.getByText('This first meeting is ungraded.')).toBeVisible()
    await page.getByRole('button', { name: 'Next' }).click()
    if (index === 0) {
      await expect.poll(() => completions).toBe(1)
      await expect(page.getByRole('button', { name: 'Next' })).toBeDisabled()
      releaseCompletion()
    }
  }
  await expect(page.getByRole('heading', { name: 'Today’s study is complete' })).toBeVisible()
  expect(completions).toBe(cards.length)
  await expect(page.getByText(/Next return:/)).toBeVisible()
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await page.getByRole('button', { name: 'Browser course · Unit 1 · meeting-1' }).click()
  const firstHistory = page.getByRole('heading', { name: "Let's get started." }).locator('..')
  await expect(firstHistory.getByRole('button', { name: 'Replay saved take' })).toHaveCount(2)
  await page.request.post('/__test/clock/2026-11-01T10:00:00Z')
  await page.reload()
  await expect(page.getByText('Independent recall')).toBeVisible()
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await page.getByRole('button', { name: 'Reveal answer' }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText("Let's get started.")
  await page.getByRole('button', { name: 'With effort' }).click()
  await expect(page.getByText('One supported repetition is available.')).toBeVisible()
  await page.getByRole('button', { name: 'Next' }).click()
  await expect(page.getByText('Independent recall')).toBeVisible()
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await page.getByLabel('Learning timezone').fill('Europe/London')
  await page.getByRole('button', { name: 'Save settings' }).click()
  await expect(page.getByText('Settings saved.')).toBeVisible()
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await page.getByLabel('Find card by source or item ID').fill('meeting-1')
  await expect(page.getByRole('button', { name: /meeting-1/ })).toHaveCount(1)
  await page.getByRole('button', { name: /meeting-1/ }).click()
  await page.getByRole('button', { name: 'Practise freely' }).click()
  await expect(page.getByText('Ungraded rehearsal')).toBeVisible()
  await page.getByRole('button', { name: 'Next' }).click()
  await expect(page.getByText('Independent recall')).toBeVisible()
})

test('restored practice card keeps the original layout and microphone interactions', async ({
  page,
}, testInfo) => {
  const visualPackage = {
    ...phrasePackage,
    package_id: 'restored-visual-card-v1',
    source_id: 'restored-visual-source',
    cards: [{ ...cards[1], item_id: 'restored-visual-card' }],
  }
  await page.goto('/')
  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(visualPackage))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByText('Added 1 phrases.')).toBeVisible()
  await page.getByRole('button', { name: 'Close' }).click()
  const card = page.locator('article.practice-card')
  await expect(card.getByRole('heading', { name: cards[1].answer })).toBeVisible()
  await expect(card.locator('.practice-prompt')).toHaveText(
    'Use the useful expression in situation 2.',
  )
  await expect(card.locator('.answer-help')).toHaveText(
    'A natural phrase for an ordinary conversation.',
  )
  await expect(card).not.toContainText('Browser course · Unit 1')
  await expect(card.locator('.microphone svg')).toBeVisible()
  expect(await card.evaluate((node) => getComputedStyle(node).borderRadius)).toBe('18px')
  expect(await card.locator('h1').evaluate((node) => getComputedStyle(node).fontFamily)).toContain(
    'Georgia',
  )
  expect(await page.locator('.shell').evaluate((node) => getComputedStyle(node).maxWidth)).toBe(
    '880px',
  )
  await testInfo.attach('restored-desktop.png', {
    body: await page.screenshot({ fullPage: true }),
    contentType: 'image/png',
  })

  await card.getByRole('button', { name: '▶ Reference' }).click()
  await expect(card.getByRole('button', { name: 'Stop playback' })).toBeVisible()
  await card.getByRole('button', { name: 'Stop playback' }).click()
  await expect(card.getByRole('button', { name: 'Record a take' })).toBeVisible()

  await page.evaluate(() => {
    const host = window as unknown as {
      __resolveMic?: (stream: unknown) => void
      __stoppedMic?: boolean
    }
    Object.defineProperty(navigator, 'mediaDevices', {
      configurable: true,
      value: {
        getUserMedia: () =>
          new Promise((resolve) => {
            host.__resolveMic = resolve
          }),
      },
    })
  })
  await card.getByRole('button', { name: 'Record a take' }).click()
  await expect(card.getByRole('button', { name: 'Cancel microphone request' })).toBeVisible()
  await card.getByRole('button', { name: 'Cancel microphone request' }).click()
  await page.evaluate(() => {
    const host = window as unknown as {
      __resolveMic: (stream: unknown) => void
      __stoppedMic?: boolean
    }
    host.__resolveMic({ getTracks: () => [{ stop: () => (host.__stoppedMic = true) }] })
  })
  await expect(card.getByRole('button', { name: 'Record a take' })).toBeVisible()
  await expect
    .poll(() =>
      page.evaluate(() => (window as unknown as { __stoppedMic?: boolean }).__stoppedMic ?? false),
    )
    .toBe(true)

  await page.setViewportSize({ width: 390, height: 844 })
  await expect(card).toBeVisible()
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
  await testInfo.attach('restored-mobile.png', {
    body: await page.screenshot({ fullPage: true }),
    contentType: 'image/png',
  })
})

test('one-time discarded-data reset completes before pending take recovery', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('button', { name: 'Add material' })).toBeVisible()
  await page.evaluate(async () => {
    localStorage.setItem('phrase-gym.reference-exposure.v1.old', 'old')
    localStorage.setItem('phrase-gym.last-learning-language.v1', 'de')
    localStorage.setItem('unrelated-app', 'keep')
    const inbox = await new Promise<IDBDatabase>((resolve, reject) => {
      const request = indexedDB.open('phrase-gym.inbox.v1', 1)
      request.onupgradeneeded = () => request.result.createObjectStore('draft')
      request.onsuccess = () => resolve(request.result)
      request.onerror = () => reject(request.error)
    })
    inbox.close()
    const db = await new Promise<IDBDatabase>((resolve, reject) => {
      const request = indexedDB.open('phrase-gym.practice.v1', 1)
      request.onsuccess = () => resolve(request.result)
      request.onerror = () => reject(request.error)
    })
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction('journal', 'readwrite')
      const journal = tx.objectStore('journal')
      journal.delete('journal-reset:original-vue-restoration-20261027-v2')
      journal.put(
        { id: 'discarded', status: 'pending', audio: new Uint8Array([1]).buffer },
        'take-v2:discarded',
      )
      tx.oncomplete = () => resolve()
      tx.onerror = () => reject(tx.error)
    })
    db.close()
  })
  let uploads = 0
  await page.route('**/api/recordings', async (route) => {
    uploads++
    await route.continue()
  })
  await page.reload()
  await expect(page.getByRole('button', { name: 'Add material' })).toBeVisible()
  expect(uploads).toBe(0)
  expect(
    await page.evaluate(async () => {
      const db = await new Promise<IDBDatabase>((resolve) => {
        const request = indexedDB.open('phrase-gym.practice.v1', 1)
        request.onsuccess = () => resolve(request.result)
      })
      const take = await new Promise<unknown>((resolve) => {
        const request = db.transaction('journal').objectStore('journal').get('take-v2:discarded')
        request.onsuccess = () => resolve(request.result)
      })
      db.close()
      return {
        take,
        exposure: localStorage.getItem('phrase-gym.reference-exposure.v1.old'),
        language: localStorage.getItem('phrase-gym.last-learning-language.v1'),
        unrelated: localStorage.getItem('unrelated-app'),
      }
    }),
  ).toEqual({ take: undefined, exposure: null, language: null, unrelated: 'keep' })
})

test('playback rejection clears the current control without stopping a newer play', async ({
  page,
}) => {
  const playable = {
    ...phrasePackage,
    package_id: 'playback-rejection-v1',
    source_id: 'playback-rejection-source',
    cards: [{ ...cards[1], item_id: 'playback-rejection-card' }],
  }
  await page.goto('/')
  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(playable))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByText('Added 1 phrases.')).toBeVisible()
  await page.getByRole('button', { name: 'Close' }).click()
  await page.getByRole('button', { name: 'Record a take' }).click()
  await page.getByRole('button', { name: 'Stop recording' }).click()
  await expect(page.getByText('Take 1', { exact: true })).toBeVisible()

  await page.evaluate(() => {
    HTMLMediaElement.prototype.play = async () => {
      throw new Error('blocked')
    }
  })
  await page.getByRole('button', { name: '▶ Reference' }).click()
  await expect(page.getByRole('alert')).toContainText('Audio playback failed')
  await expect(page.getByRole('button', { name: 'Record a take' })).toBeVisible()
  await page.getByRole('button', { name: 'Compare with reference' }).click()
  await expect(page.getByRole('alert')).toContainText('Audio playback failed')
  await expect(page.getByRole('button', { name: 'Record a take' })).toBeVisible()

  await page.evaluate(() => {
    const host = window as unknown as { __rejectOldPlay?: () => void; __playCalls?: number }
    host.__playCalls = 0
    HTMLMediaElement.prototype.play = function () {
      host.__playCalls!++
      if (host.__playCalls === 1)
        return new Promise<void>((_resolve, reject) => {
          host.__rejectOldPlay = () => reject(new Error('old playback blocked'))
        })
      return Promise.resolve()
    }
  })
  await page.getByRole('button', { name: '▶ Reference' }).click()
  await page.getByRole('button', { name: '▶ Reference' }).click()
  await page.evaluate(() =>
    (window as unknown as { __rejectOldPlay: () => void }).__rejectOldPlay(),
  )
  await expect(page.getByRole('button', { name: 'Stop playback' })).toBeVisible()
  await expect(page.getByRole('alert')).toHaveCount(0)
})

test('import modal traps keyboard focus and finishes recording before opening', async ({
  page,
}) => {
  const modalPackage = {
    ...phrasePackage,
    package_id: 'modal-recording-v1',
    source_id: 'modal-recording-source',
    cards: [{ ...cards[1], item_id: 'modal-recording-card' }],
  }
  await page.goto('/')
  const opener = page.getByRole('button', { name: 'Add material' })
  await opener.click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(modalPackage))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByText('Added 1 phrases.')).toBeVisible()
  await page.getByRole('button', { name: 'Close' }).click()
  await page.getByRole('button', { name: 'Record a take' }).click()
  await expect(page.getByRole('button', { name: 'Stop recording' })).toBeVisible()
  await opener.click()
  const dialog = page.getByRole('dialog', { name: 'Add material' })
  await expect(dialog).toBeVisible()
  await expect(page.getByText('Take 1', { exact: true })).toBeVisible()
  expect(
    await page.evaluate(
      () =>
        (window as unknown as { __playedMedia?: HTMLMediaElement[] }).__playedMedia?.length ?? 0,
    ),
  ).toBe(0)
  await expect(page.getByRole('button', { name: 'Record a take' })).toBeVisible()
  await dialog.getByRole('button', { name: 'Close' }).focus()
  await page.keyboard.press('Tab')
  expect(
    await page.evaluate(() => document.querySelector('dialog')?.contains(document.activeElement)),
  ).toBe(true)
  await page.keyboard.press('Escape')
  await expect(dialog).not.toBeVisible()
  await expect(opener).toBeFocused()
})

test('automatic comparison can be stopped while upload is pending and late end cannot restart it', async ({
  page,
}) => {
  const comparisonPackage = {
    ...phrasePackage,
    package_id: 'automatic-comparison-stop-v1',
    source_id: 'automatic-comparison-stop-source',
    cards: [{ ...cards[1], item_id: 'automatic-comparison-stop-card' }],
  }
  await page.goto('/')
  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(comparisonPackage))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(page.getByText('Added 1 phrases.')).toBeVisible()
  await page.getByRole('button', { name: 'Close' }).click()

  await page.evaluate(() => {
    class SilentAudio {
      src = ''
      ended = false
      onended: (() => void) | null = null
      onpause: (() => void) | null = null
      onerror: (() => void) | null = null
      play() {
        const host = window as unknown as { __playedMedia?: SilentAudio[] }
        ;(host.__playedMedia ??= []).push(this)
        return Promise.resolve()
      }
      pause() {
        this.onpause?.()
      }
      dispatchEvent(event: Event) {
        if (event.type === 'ended') this.onended?.()
        if (event.type === 'pause') this.onpause?.()
        return true
      }
    }
    Object.defineProperty(window, 'Audio', { configurable: true, value: SilentAudio })
  })

  let releaseUpload!: () => void
  const uploadHeld = new Promise<void>((resolve) => {
    releaseUpload = resolve
  })
  await page.route('**/api/recordings', async (route) => {
    if (route.request().method() === 'POST') await uploadHeld
    await route.continue()
  })
  await page.getByRole('button', { name: 'Record a take' }).click()
  await page.getByRole('button', { name: 'Stop recording' }).click()
  await expect(page.getByText('Finishing and saving the complete take…')).toBeVisible()
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          (window as unknown as { __playedMedia?: HTMLMediaElement[] }).__playedMedia?.length ?? 0,
      ),
    )
    .toBe(1)
  const stop = page.getByRole('button', { name: 'Stop playback' })
  await expect(stop).toBeEnabled()
  await stop.click()
  await page.evaluate(() => {
    const media = (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia[0]!
    media.dispatchEvent(new Event('ended'))
  })
  expect(
    await page.evaluate(
      () => (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.length,
    ),
  ).toBe(1)
  releaseUpload()
  await expect(page.locator('article.study').getByText('Take 1', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: '▶ Reference' }).click()
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.length,
      ),
    )
    .toBe(2)
  await page.evaluate(() => {
    const media = (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia[0]!
    media.dispatchEvent(new Event('ended'))
  })
  expect(
    await page.evaluate(
      () => (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.length,
    ),
  ).toBe(2)
})
