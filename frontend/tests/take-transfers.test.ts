import { beforeEach, expect, test, vi } from 'vitest'

const state = vi.hoisted(() => ({
  acknowledged: null as ArrayBuffer | null,
  body: null as Record<string, unknown> | null,
}))

const partial = {
  id: 'take-race',
  phrase_id: 'phrase-race',
  attempt_id: 'attempt-race',
  captured_at: '2026-10-26T08:00:00Z',
  mime_type: 'audio/webm',
  duration_ms: 100,
  complete: false,
  capture_settings: {},
  audio: new Uint8Array([1]).buffer,
  status: 'pending' as const,
}
const final = {
  ...partial,
  duration_ms: 500,
  complete: true,
  audio: new Uint8Array([1, 2, 3, 4]).buffer,
}

vi.mock('../src/captureOwners', () => ({
  releaseCapture: vi.fn(),
  withAvailableTake: async (_id: string, operation: () => Promise<void>) => {
    await operation()
    return true
  },
}))
vi.mock('../src/practiceJournal', () => ({
  journalPut: vi.fn(),
  journalGet: async () => final,
}))
vi.mock('../src/takeJournalRecovery', () => ({
  takePrefix: 'take-v2:',
  recoverTake: vi.fn(),
  takeEntries: async () => [{ key: 'take-v2:take-race', value: partial }],
  acknowledgeTake: async (_key: string, take: typeof final) => {
    state.acknowledged = take.audio
  },
}))
vi.mock('../src/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../src/api')>()),
  api: async (_path: string, init: RequestInit) => {
    state.body = JSON.parse(String(init.body)) as Record<string, unknown>
    const bytes = final.audio
    const digest = await crypto.subtle.digest('SHA-256', bytes)
    return {
      id: final.id,
      byte_length: bytes.byteLength,
      sha256: Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, '0')).join(
        '',
      ),
      complete: true,
    }
  },
}))

beforeEach(() => {
  state.acknowledged = null
  state.body = null
})

test('re-reads the finalized journal row after acquiring take ownership', async () => {
  const { takeTransfers } = await import('../src/takeTransfers')
  await takeTransfers.flush()
  expect(state.body).toMatchObject({
    id: 'take-race',
    duration_ms: 500,
    complete: true,
    audio_base64: 'AQIDBA==',
  })
  expect(new Uint8Array(state.acknowledged!)).toEqual(new Uint8Array([1, 2, 3, 4]))
})
