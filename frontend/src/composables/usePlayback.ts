import { onUnmounted, ref } from 'vue'
import { setAudioSession } from '../audioSession'
import type { Recording } from '../studyTypes'

export function usePlayback() {
  let playing: HTMLAudioElement | null = null
  const active = ref(false)
  const error = ref('')

  function stop() {
    if (playing) {
      const current = playing
      playing = null
      current.pause()
      current.src = ''
    }
    active.value = false
    setAudioSession('auto')
  }

  async function play(url: string) {
    stop()
    error.value = ''
    setAudioSession('playback')
    const audio = new Audio(url)
    playing = audio
    active.value = true
    audio.onended = () => {
      if (playing === audio) {
        playing = null
        active.value = false
        setAudioSession('auto')
      }
    }
    audio.onerror = () => {
      if (playing !== audio) return
      stop()
      error.value = 'Audio playback failed.'
    }
    audio.onpause = () => {
      if (playing !== audio) return
      playing = null
      active.value = false
      setAudioSession('auto')
    }
    try {
      await audio.play()
    } catch (cause) {
      if (playing !== audio) return
      stop()
      error.value = `Audio playback failed: ${String(cause)}`
    }
  }

  async function compare(recording: Recording, referenceUrl?: string) {
    stop()
    error.value = ''
    setAudioSession('playback')
    const learner = new Audio(recording.url)
    playing = learner
    active.value = true
    learner.onended = () => {
      if (playing !== learner || !referenceUrl) return
      void play(referenceUrl)
    }
    learner.onpause = () => {
      if (playing !== learner) return
      learner.src = ''
      playing = null
      active.value = false
      setAudioSession('auto')
    }
    learner.onerror = () => {
      if (playing !== learner) return
      stop()
      error.value = 'Audio playback failed.'
    }
    try {
      await learner.play()
    } catch (cause) {
      if (playing !== learner) return
      stop()
      error.value = `Audio playback failed: ${String(cause)}`
    }
  }

  onUnmounted(stop)
  return { play, compare, stop, active, error }
}
