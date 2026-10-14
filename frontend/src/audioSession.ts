type SessionType = 'auto' | 'playback' | 'play-and-record'

export function setAudioSession(type: SessionType) {
  const session = (navigator as Navigator & { audioSession?: { type: string } }).audioSession
  if (!session) return
  try {
    session.type = type
  } catch {
    // Browsers without a writable Audio Session API manage routing themselves.
  }
}
