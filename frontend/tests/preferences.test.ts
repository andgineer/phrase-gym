import { afterEach, expect, test, vi } from 'vitest'
import { ref } from 'vue'
import { usePreferences } from '../src/composables/usePreferences'
import { serverUnavailable, connectionRetrying } from '../src/api'
import { rememberLanguage, restoredLanguage } from '../src/learningLanguage'

afterEach(() => {
  vi.unstubAllGlobals()
  serverUnavailable.value = false
  connectionRetrying.value = false
})

test('device language validates storage and tolerates failed reads and writes', () => {
  const storage = { getItem: vi.fn(() => 'sr'), setItem: vi.fn() }
  vi.stubGlobal('localStorage', storage)
  expect(restoredLanguage()).toBe('sr')
  storage.getItem.mockReturnValue('invalid')
  expect(restoredLanguage()).toBeNull()
  storage.getItem.mockImplementation(() => {
    throw new Error('Blocked')
  })
  storage.setItem.mockImplementation(() => {
    throw new Error('Full')
  })
  expect(restoredLanguage()).toBeNull()
  expect(() => rememberLanguage('en')).not.toThrow()
})

test('simultaneous Settings submissions send one immutable payload and a server error blocks new saves until retry', async () => {
  let release!: (response: Response) => void
  const fetch = vi.fn(
    (_input: RequestInfo | URL, _init?: RequestInit) =>
      new Promise<Response>((resolve) => {
        release = resolve
      }),
  )
  vi.stubGlobal('fetch', fetch)
  const preferences = usePreferences(ref('en'))
  preferences.native.value = 'ru'
  preferences.configuring.value = true
  const pending = preferences.save()
  preferences.native.value = 'de'
  expect(await preferences.save()).toBe(false)
  expect(fetch).toHaveBeenCalledTimes(1)
  expect(JSON.parse(String(fetch.mock.calls[0]![1]?.body))).toEqual({
    native_language: 'ru',
    ui_language_mode: 'native',
  })
  release(new Response('{"detail":"Unavailable"}', { status: 503 }))
  await expect(pending).rejects.toThrow('Unavailable')
  expect(preferences.savingSettings.value).toBe(false)
  expect(preferences.configuring.value).toBe(true)
  await expect(preferences.save()).rejects.toThrow('Server connection required')
  expect(fetch).toHaveBeenCalledTimes(1)
  serverUnavailable.value = false
  const refresh = preferences.load(true)
  release(new Response('{"native_language":"ru","ui_language_mode":"native"}'))
  await refresh
  expect(preferences.native.value).toBe('de')
  const retry = preferences.save()
  release(new Response('{"native_language":"de","ui_language_mode":"native"}'))
  expect(await retry).toBe(true)
  expect(preferences.settings.value?.native_language).toBe('de')
  expect(preferences.configuring.value).toBe(false)
})
