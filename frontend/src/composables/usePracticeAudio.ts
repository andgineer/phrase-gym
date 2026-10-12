import { onMounted, onUnmounted, ref } from 'vue'

export function usePracticeAudio() {
  const playing = ref(false)
  const error = ref('')
  let audio: HTMLAudioElement | null = null
  let sequence = 0
  let lastUrl = ''
  let resolvePlayback: (() => void) | null = null
  function stop() {
    sequence++
    audio?.pause()
    if (audio) audio.src = ''
    audio = null
    playing.value = false
    resolvePlayback?.()
    resolvePlayback = null
  }
  async function play(url: string): Promise<boolean> {
    stop()
    lastUrl = url
    error.value = ''
    const token = sequence
    const playback = new Audio(url)
    audio = playback
    playing.value = true
    let successful = true
    const ended = new Promise<void>((resolve) => {
      resolvePlayback = resolve
    })
    playback.onended = () => {
      if (audio === playback) resolvePlayback?.()
    }
    playback.onerror = () => {
      if (audio === playback) {
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
      if (audio === playback) error.value = 'Audio could not play. Try again.'
    }
    if (token !== sequence) return false
    stop()
    return successful
  }
  function background() {
    if (document.hidden) stop()
  }
  onMounted(() => document.addEventListener('visibilitychange', background))
  onUnmounted(() => {
    stop()
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
