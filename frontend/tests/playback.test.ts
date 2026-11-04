import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import { usePlayback } from '../src/composables/usePlayback'
import type { StoredTake } from '../src/practiceTypes'

vi.mock('vue', async (original) => ({
  ...(await original<typeof import('vue')>()),
  onUnmounted: vi.fn(),
}))

class AudioStub {
  static played: AudioStub[] = []
  onended: (() => void) | null = null
  onpause: (() => void) | null = null
  onerror: (() => void) | null = null
  ended = false
  constructor(public src: string) {}
  play() {
    AudioStub.played.push(this)
    return Promise.resolve()
  }
  pause() {
    this.onpause?.()
  }
}
const take = {
  audio: new Uint8Array([1, 2]).buffer,
  mime_type: 'audio/webm',
  duration_ms: 2000,
} as StoredTake

beforeEach(() => {
  vi.useFakeTimers()
  AudioStub.played = []
  vi.stubGlobal('Audio', AudioStub)
  vi.stubGlobal('window', globalThis)
  vi.stubGlobal('document', { addEventListener: vi.fn(), removeEventListener: vi.fn() })
  vi.stubGlobal('navigator', {})
  vi.spyOn(URL, 'createObjectURL').mockReturnValue('blob:take')
  vi.spyOn(URL, 'revokeObjectURL').mockImplementation(() => {})
})
afterEach(() => {
  vi.useRealTimers()
  vi.restoreAllMocks()
  vi.unstubAllGlobals()
})

it('keeps the local take available through the reference and releases it after the learner ends', async () => {
  const playback = usePlayback()
  await playback.compareTake(take, '/reference')
  const reference = AudioStub.played[0]!
  expect(reference.src).toBe('/reference')
  await vi.advanceTimersByTimeAsync(10_000)
  expect(AudioStub.played).toHaveLength(1)
  reference.onended?.()
  const learner = AudioStub.played[1]!
  expect(learner.src).toBe('blob:take')
  expect(URL.revokeObjectURL).not.toHaveBeenCalled()
  reference.onended?.()
  expect(AudioStub.played).toHaveLength(2)
  learner.onended?.()
  expect(playback.active.value).toBe(false)
  expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:take')
})

it('uses the recorded duration fallback only for the learner clip', async () => {
  const playback = usePlayback()
  await playback.compareTake(take, '/reference')
  AudioStub.played[0]!.onended?.()
  await vi.advanceTimersByTimeAsync(2999)
  expect(playback.active.value).toBe(true)
  await vi.advanceTimersByTimeAsync(1)
  expect(playback.active.value).toBe(false)
  expect(AudioStub.played).toHaveLength(2)
  expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:take')
})

it.each(['pause', 'error', 'stop'] as const)(
  '%s during the reference cancels the pending learner clip',
  async (action) => {
    const playback = usePlayback()
    await playback.compareTake(take, '/reference')
    const reference = AudioStub.played[0]!
    if (action === 'pause') reference.pause()
    else if (action === 'error') reference.onerror?.()
    else playback.stop()
    reference.onended?.()
    await vi.advanceTimersByTimeAsync(10_000)
    expect(AudioStub.played).toHaveLength(1)
    expect(playback.active.value).toBe(false)
    expect(URL.revokeObjectURL).toHaveBeenCalledWith('blob:take')
    expect(playback.error.value).toBe(action === 'error' ? 'Audio playback failed.' : '')
  },
)

it('new playback cancels the old learner fallback and late events', async () => {
  const playback = usePlayback()
  await playback.compareTake(take)
  const learner = AudioStub.played[0]!
  await playback.play('/another-reference')
  learner.onended?.()
  learner.onerror?.()
  await vi.advanceTimersByTimeAsync(10_000)
  expect(AudioStub.played).toHaveLength(2)
  expect(AudioStub.played[1]!.src).toBe('/another-reference')
  expect(playback.active.value).toBe(true)
  expect(playback.error.value).toBe('')
  playback.stop()
})
