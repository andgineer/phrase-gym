import { beforeEach, expect, test, vi } from 'vitest'
import { ApiError } from '../src/api'

const state = vi.hoisted(() => ({
  failure: null as Error | null,
  deleted: [] as string[],
  entries: [] as Array<{ key: string; value: Record<string, unknown> }>,
}))

const take = {
  id: 'take-for-deleted-phrase',
  phrase_id: 'deleted-phrase',
  attempt_id: 'attempt',
  captured_at: '2026-10-26T08:00:00Z',
  mime_type: 'audio/webm',
  duration_ms: 100,
  complete: true,
  capture_settings: {},
  audio: new Uint8Array([1]).buffer,
  status: 'pending' as const,
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
  journalGet: async () => take,
  journalDelete: async (key: string) => {
    state.deleted.push(key)
    state.entries = state.entries.filter((entry) => entry.key !== key)
  },
}))
vi.mock('../src/takeJournalRecovery', () => ({
  takePrefix: 'take-v2:',
  recoverTake: vi.fn(),
  takeEntries: async () => state.entries,
  acknowledgeTake: vi.fn(),
}))
vi.mock('../src/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../src/api')>()),
  api: async () => {
    throw state.failure
  },
}))

beforeEach(() => {
  state.deleted = []
  state.entries = [{ key: `take-v2:${take.id}`, value: take }]
})

test.each([410, 409, 413, 422])(
  'a permanent %s refusal is kept until discarded',
  async (status) => {
    const { takeTransfers } = await import('../src/takeTransfers')
    state.failure = new ApiError('This phrase was deleted. Discard it from this device.', status)
    await takeTransfers.flush()
    expect(takeTransfers.rejected.value).toEqual([
      {
        id: take.id,
        key: `take-v2:${take.id}`,
        message: 'This phrase was deleted. Discard it from this device.',
      },
    ])
    expect(takeTransfers.error.value).toBe('')
    expect(takeTransfers.pendingTakes.value).toBe(1)
    expect(state.deleted).toEqual([])

    await takeTransfers.discard(take.id)
    expect(state.deleted).toEqual([`take-v2:${take.id}`])
    expect(takeTransfers.rejected.value).toEqual([])
    expect(takeTransfers.pendingTakes.value).toBe(0)
  },
)

test.each([
  [503, 'Temporary save failure'],
  [403, 'Cross-origin mutations are not allowed'],
  [400, 'Invalid host header'],
])('a %s refusal stays retryable and offers no discard', async (status, message) => {
  const { takeTransfers } = await import('../src/takeTransfers')
  state.failure = new ApiError(message, status)
  await takeTransfers.flush()
  expect(takeTransfers.rejected.value).toEqual([])
  expect(takeTransfers.error.value).toContain('Audio remains on this device')
  await takeTransfers.discard(take.id)
  expect(state.deleted).toEqual([])
})
