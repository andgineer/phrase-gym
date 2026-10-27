import 'fake-indexeddb/auto'
import { expect, test, vi } from 'vitest'
import { resetDiscardedLearnerDataOnce } from '../src/discardedLearnerData'
import { journalEntries, journalGet, journalPut } from '../src/practiceJournal'

test('clears known old learner stores once, then preserves new takes and unrelated origin data', async () => {
  const values = new Map([
    ['phrase-gym.reference-exposure.v1.old', 'old exposure'],
    ['phrase-gym.last-learning-language.v1', 'de'],
    ['unrelated-app', 'keep'],
  ])
  vi.stubGlobal('localStorage', {
    getItem: (key: string) => values.get(key) ?? null,
    setItem: (key: string, value: string) => values.set(key, value),
    removeItem: (key: string) => values.delete(key),
    key: (index: number) => [...values.keys()][index] ?? null,
    get length() {
      return values.size
    },
  })

  const inbox = await new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open('phrase-gym.inbox.v1', 1)
    request.onupgradeneeded = () => request.result.createObjectStore('draft')
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
  await new Promise<void>((resolve, reject) => {
    const tx = inbox.transaction('draft', 'readwrite')
    tx.objectStore('draft').put({ old: 'draft' }, 'current')
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
  })
  inbox.close()
  await journalPut('take-v2:discarded', { audio: new Uint8Array([1]).buffer })
  await journalPut('legacy-checkpoint', { card: 'discarded' })

  await resetDiscardedLearnerDataOnce()
  expect(await journalEntries('take-v2:')).toEqual([])
  expect(await journalGet('legacy-checkpoint')).toBeUndefined()
  expect(values.has('phrase-gym.reference-exposure.v1.old')).toBe(false)
  expect(values.has('phrase-gym.last-learning-language.v1')).toBe(false)
  expect(values.get('unrelated-app')).toBe('keep')
  const recreatedInbox = await new Promise<IDBDatabase>((resolve, reject) => {
    const request = indexedDB.open('phrase-gym.inbox.v1', 1)
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
  expect([...recreatedInbox.objectStoreNames]).toEqual([])
  recreatedInbox.close()

  const pending = { audio: new Uint8Array([2, 3]).buffer }
  await journalPut('take-v2:new', pending)
  values.set('phrase-gym.reference-exposure.v1.new', 'keep after reset')
  await resetDiscardedLearnerDataOnce()
  expect(await journalGet('take-v2:new')).toEqual(pending)
  expect(values.get('phrase-gym.reference-exposure.v1.new')).toBe('keep after reset')
})
