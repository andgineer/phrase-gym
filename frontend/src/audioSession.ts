type SessionType = 'auto' | 'playback' | 'play-and-record'
const microphones = new Set<MediaStream>()

export function retainMicrophone(stream: MediaStream): () => void {
  microphones.add(stream)
  return () => microphones.delete(stream)
}

export function setAudioSession(type: SessionType) {
  const session = (navigator as Navigator & { audioSession?: { type: string } }).audioSession
  if (!session) return
  try {
    // Playback-only sessions can end iOS microphone tracks retained between takes.
    const hasMicrophone = [...microphones].some((stream) =>
      stream.getAudioTracks().some((track) => track.readyState === 'live'),
    )
    session.type = hasMicrophone ? 'play-and-record' : type
  } catch {
    // Browsers without a writable Audio Session API manage routing themselves.
  }
}
