type SessionType = 'auto' | 'playback' | 'play-and-record'
const microphones = new Set<MediaStream>()

export function retainMicrophone(stream: MediaStream): () => void {
  microphones.add(stream)
  return () => microphones.delete(stream)
}

// iOS capture fails with processing forced off; desktop keeps the original unprocessed capture.
export function microphoneConstraints(): MediaStreamConstraints {
  if (navigator.maxTouchPoints > 0) return { audio: true }
  return { audio: { echoCancellation: false, noiseSuppression: false, autoGainControl: false } }
}

// Reopening a desktop input switches Bluetooth headsets between call and music profiles,
// which silences playback midway through comparison.
export function retainsMicrophoneBetweenTakes(): boolean {
  return navigator.maxTouchPoints === 0
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
