import { IDBFactory } from 'fake-indexeddb'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { inboxDraft, type InboxDraft } from '../src/inboxDraft'

beforeEach(() => vi.stubGlobal('indexedDB', new IDBFactory()))
afterEach(() => vi.unstubAllGlobals())
function input(id: string): InboxDraft {
  return {
    submission: {
      submission_id: id,
      item_id: id,
      expected_generation: null,
      language: 'en',
      title: id,
      context: 'Synthetic context',
      note: '',
      phrases: [
        {
          source_id: 'phrase',
          text: id,
          reference_audio_id: null,
          audio_sha256: null,
          expected_revision: null,
        },
      ],
    },
  }
}

test('concurrent empty-slot claims retain exactly one complete identity and its bytes', async () => {
  const a = input('A')
  const b = input('B')
  const results = await Promise.all([inboxDraft(a), inboxDraft(b)])
  const winner = await inboxDraft()
  expect([a, b]).toContainEqual(winner)
  expect(results).toEqual([winner, winner])
  expect(await inboxDraft(input('C'))).toEqual(winner)
})

test('same-identity late updates preserve exact original text and durable acknowledgement', async () => {
  const original = input('A')
  await inboxDraft(original)
  const uploaded = structuredClone(original)
  await inboxDraft(uploaded, 'A')
  const acknowledged = structuredClone(uploaded)
  acknowledged.receipt = {
    generation_id: 'generation',
    submission_id: 'A',
    generation: 1,
    namespace: 'test',
    profile_revision: 1,
    guide_version: 'test',
    received_at: '2026-10-19T10:00:00Z',
  }
  await inboxDraft(acknowledged, 'A')
  expect(await inboxDraft(original, 'A')).toEqual(acknowledged)
  expect(await inboxDraft(uploaded, 'A')).toEqual(acknowledged)
})

test('late updates cannot recreate abandoned input or overwrite/delete its replacement', async () => {
  const a = input('A')
  const b = input('B')
  await inboxDraft(a)
  await inboxDraft(null, 'A')
  expect(await inboxDraft(a, 'A')).toBeUndefined()
  await inboxDraft(b)
  expect(await inboxDraft(a, 'A')).toEqual(b)
  expect(await inboxDraft(null, 'A')).toEqual(b)
  expect(await inboxDraft(null)).toEqual(b)
  expect(await inboxDraft()).toEqual(b)
  expect(await inboxDraft(null, 'B')).toBeUndefined()
})
