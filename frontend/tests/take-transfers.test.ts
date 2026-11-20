import { beforeEach, expect, test, vi } from 'vitest'
import { ApiError } from '../src/api'
import type { StoredTake } from '../src/practiceTypes'

const state = vi.hoisted(() => ({
  rows: new Map<string, StoredTake>(),
  beforeLock: vi.fn<() => void>(),
  post: vi.fn<(body: { id: string; audio_base64: string; complete: boolean }) => Promise<void>>(),
}))

const take: StoredTake = {
  id: 'take-race',
  phrase_id: 'phrase-race',
  view_token: 'view-race',
  mode: 'daily',
  kind: 'recall',
  cue: { situation: 'Colleague asks: How is it going?' },
  hint: 'far',
  captured_at: '2026-10-26T08:00:00Z',
  mime_type: 'audio/webm',
  duration_ms: 100,
  complete: false,
  capture_settings: {},
  audio: new Uint8Array([1]).buffer,
  status: 'pending',
}

vi.mock('../src/captureOwners', () => ({
  releaseCapture: vi.fn(),
  withAvailableTake: async (_id: string, operation: () => Promise<void>) => {
    state.beforeLock()
    await operation()
    return true
  },
}))
vi.mock('../src/practiceJournal', () => ({
  journalPut: async (key: string, value: StoredTake) => {
    state.rows.set(key, value)
  },
  journalGet: async (key: string) => state.rows.get(key),
}))
vi.mock('../src/takeJournalRecovery', () => ({
  takePrefix: 'take-v2:',
  recoverTake: vi.fn(),
  takeEntries: async () => Array.from(state.rows, ([key, value]) => ({ key, value })),
  acknowledgeTake: async (key: string, uploaded: StoredTake) => {
    expect(uploaded).toBe(state.rows.get(key))
    state.rows.delete(key)
  },
}))
vi.mock('../src/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../src/api')>()),
  api: async (_path: string, init: RequestInit) => {
    const body = JSON.parse(String(init.body))
    await state.post(body)
    const audio = Uint8Array.from(atob(body.audio_base64), (character) => character.charCodeAt(0))
    const digest = await crypto.subtle.digest('SHA-256', audio)
    return {
      id: body.id,
      byte_length: audio.byteLength,
      sha256: Array.from(new Uint8Array(digest), (byte) => byte.toString(16).padStart(2, '0')).join(
        '',
      ),
      complete: body.complete,
    }
  },
}))

beforeEach(() => {
  vi.resetModules()
  state.rows.clear()
  state.beforeLock.mockReset()
  state.post.mockReset()
})

function holdPost() {
  let report!: (id: string) => void
  let release!: () => void
  const requested = new Promise<string>((resolve) => (report = resolve))
  const held = new Promise<void>((resolve) => (release = resolve))
  return {
    requested,
    release,
    wait: ({ id }: { id: string }) => {
      report(id)
      return held
    },
  }
}

test('re-reads the finalized journal row after acquiring take ownership', async () => {
  const key = `take-v2:${take.id}`
  state.rows.set(key, take)
  state.beforeLock.mockImplementationOnce(() => {
    state.rows.set(key, {
      ...take,
      duration_ms: 500,
      complete: true,
      audio: new Uint8Array([1, 2, 3, 4]).buffer,
    })
  })
  const { takeTransfers } = await import('../src/takeTransfers')
  await takeTransfers.flush()
  expect(state.post).toHaveBeenCalledExactlyOnceWith(
    expect.objectContaining({
      id: 'take-race',
      view_token: 'view-race',
      mode: 'daily',
      kind: 'recall',
      cue: { situation: 'Colleague asks: How is it going?' },
      hint: 'far',
      duration_ms: 500,
      complete: true,
      audio_base64: 'AQIDBA==',
    }),
  )
  expect(state.rows.has(key)).toBe(false)
})

test('a completed take queued during another upload gets its own upload', async () => {
  const firstPost = holdPost()
  state.post.mockImplementationOnce(firstPost.wait)

  const { takeTransfers } = await import('../src/takeTransfers')
  await takeTransfers.retain(take)
  const first = takeTransfers.flush()
  await firstPost.requested
  const secondTake = { ...take, id: 'second-take' }
  await takeTransfers.retain({ ...secondTake, status: 'capturing' })
  expect(takeTransfers.pendingUploads.value).toBe(1)
  expect(takeTransfers.failedCount.value).toBe(0)
  expect(takeTransfers.retrying.value).toBe(false)

  await takeTransfers.retain(secondTake)
  const second = takeTransfers.flush()
  expect(takeTransfers.pendingUploads.value).toBe(2)
  expect(state.post).toHaveBeenCalledTimes(1)
  firstPost.release()
  await Promise.all([first, second])

  expect(state.post.mock.calls.map(([body]) => body.id)).toEqual([take.id, secondTake.id])
  expect(takeTransfers.savedRevision.value).toBe(2)
  expect(takeTransfers.pendingUploads.value).toBe(0)
})

test.each(['succeeds', 'fails'] as const)(
  'a later initial upload stays quiet after the current retry %s',
  async (result) => {
    const { takeTransfers } = await import('../src/takeTransfers')
    const secondTake = { ...take, id: 'second-take' }
    const retryPost = holdPost()
    const secondPost = holdPost()
    state.post
      .mockRejectedValueOnce(new ApiError('Outage', 503))
      .mockImplementationOnce(async (body) => {
        await retryPost.wait(body)
        if (result === 'fails') throw new ApiError('Outage', 503)
      })
      .mockImplementationOnce(secondPost.wait)
    await takeTransfers.retain(take)
    await takeTransfers.flush()

    const retry = takeTransfers.retry()
    expect(await retryPost.requested).toBe(take.id)
    expect(takeTransfers.retrying.value).toBe(true)
    await takeTransfers.retain(secondTake)
    const nextFlush = takeTransfers.flush()
    retryPost.release()

    let anotherRetry: ReturnType<typeof takeTransfers.retry>
    try {
      expect(await secondPost.requested).toBe(secondTake.id)
      expect(takeTransfers.retrying.value).toBe(false)
      expect(takeTransfers.failedCount.value).toBe(result === 'fails' ? 1 : 0)
      if (result === 'fails') anotherRetry = takeTransfers.retry()
    } finally {
      secondPost.release()
      await Promise.all([retry, nextFlush, anotherRetry])
    }

    const attempts = [take.id, take.id, secondTake.id]
    if (result === 'fails') attempts.push(take.id)
    expect(state.post.mock.calls.map(([body]) => body.id)).toEqual(attempts)
    expect(takeTransfers.pendingUploads.value).toBe(0)
  },
)
