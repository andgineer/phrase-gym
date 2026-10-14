import { afterEach, expect, test, vi } from 'vitest'
import { retainMicrophone, setAudioSession } from '../src/audioSession'

afterEach(() => vi.unstubAllGlobals())

test('playback and auto preserve live microphones until every owner releases them', () => {
  const session = { type: 'auto' }
  vi.stubGlobal('navigator', { audioSession: session })
  const stream = () => ({ getAudioTracks: () => [{ readyState: 'live' }] }) as MediaStream
  const first = retainMicrophone(stream())
  const second = retainMicrophone(stream())
  try {
    setAudioSession('playback')
    expect(session.type).toBe('play-and-record')
    first()
    setAudioSession('auto')
    expect(session.type).toBe('play-and-record')
    second()
    setAudioSession('playback')
    expect(session.type).toBe('playback')
    setAudioSession('auto')
    expect(session.type).toBe('auto')
  } finally {
    first()
    second()
  }
})

test('ended microphone tracks no longer reserve the recording session', () => {
  const session = { type: 'auto' }
  vi.stubGlobal('navigator', { audioSession: session })
  const track = { readyState: 'live' }
  const release = retainMicrophone({ getAudioTracks: () => [track] } as MediaStream)
  try {
    setAudioSession('playback')
    expect(session.type).toBe('play-and-record')
    track.readyState = 'ended'
    setAudioSession('playback')
    expect(session.type).toBe('playback')
  } finally {
    release()
  }
})

test('missing or read-only audio session APIs leave native routing available', () => {
  vi.stubGlobal('navigator', {})
  expect(() => setAudioSession('play-and-record')).not.toThrow()
  vi.stubGlobal('navigator', { audioSession: Object.freeze({ type: 'auto' }) })
  expect(() => setAudioSession('play-and-record')).not.toThrow()
})
