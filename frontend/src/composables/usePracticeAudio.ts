import { onMounted, onUnmounted, ref } from 'vue'
import { requireServer } from '../api'
import { setAudioSession } from '../audioSession'

export function usePracticeAudio() {
  const playing = ref(false)
  const error = ref('')
  let audio: HTMLAudioElement | null = null
  let sequence = 0
  let lastUrl = ''
  let resolvePlayback: (() => void) | null = null
  let live = true
  function stop() {
    sequence++
    audio?.pause()
    if (audio) {
      audio.onended = null
      audio.onerror = null
      audio.src = ''
    }
    playing.value = false
    resolvePlayback?.()
    resolvePlayback = null
  }
  async function play(url: string): Promise<boolean> {
    if (!live) return false
    try {
      requireServer()
    } catch {
      error.value = 'Server connection required before playback'
      return false
    }
    stop()
    lastUrl = url
    error.value = ''
    const token = sequence
    const playback = (audio ??= new Audio())
    playback.src = url
    setAudioSession('playback')
    playing.value = true
    let successful = true
    const ended = new Promise<void>((resolve) => {
      resolvePlayback = resolve
    })
    playback.onended = () => {
      if (token === sequence) resolvePlayback?.()
    }
    playback.onerror = () => {
      if (token === sequence) {
        successful = false
        error.value = 'Audio could not play. Try again.'
        resolvePlayback?.()
      }
    }
    try {
      await playback.play()
      await ended
    } catch {
      successful = false
      if (token === sequence) error.value = 'Audio could not play. Try again.'
    }
    if (token !== sequence) return false
    stop()
    return successful
  }
  function background() {
    if (document.hidden) {
      stop()
      setAudioSession('auto')
    }
  }
  onMounted(() => document.addEventListener('visibilitychange', background))
  onUnmounted(() => {
    live = false
    stop()
    setAudioSession('auto')
    audio = null
    document.removeEventListener('visibilitychange', background)
  })
  return {
    playing,
    error,
    play,
    stop,
    retry: () => (lastUrl ? play(lastUrl) : Promise.resolve(false)),
  }
}
