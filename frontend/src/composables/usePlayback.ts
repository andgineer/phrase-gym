import { onUnmounted, ref } from 'vue'
import { setAudioSession } from '../audioSession'
import type { StoredTake } from '../practiceTypes'
import type { Recording } from '../studyTypes'

export function usePlayback() {
  let playing: HTMLAudioElement | null = null
  const active = ref(false)
  const error = ref('')
  let localUrl: string | null = null
  let comparisonTimer: number | null = null

  function stop() {
    if (comparisonTimer !== null) window.clearTimeout(comparisonTimer)
    comparisonTimer = null
    if (playing) {
      const current = playing
      playing = null
      current.pause()
      current.src = ''
    }
    active.value = false
    if (localUrl) URL.revokeObjectURL(localUrl)
    localUrl = null
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

  async function compareUrl(url: string, referenceUrl?: string, durationMs?: number | null) {
    error.value = ''
    setAudioSession('playback')
    const learner = new Audio(url)
    playing = learner
    active.value = true
    const advance = () => {
      if (playing !== learner) return
      if (comparisonTimer !== null) window.clearTimeout(comparisonTimer)
      comparisonTimer = null
      playing = null
      if (!referenceUrl) return stop()
      void play(referenceUrl)
    }
    learner.onended = advance
    // Some browser WebM encoders omit duration metadata and never signal the final frame.
    learner.onpause = () => {
      if (playing === learner && !learner.ended) stop()
    }
    learner.onerror = () => {
      if (playing !== learner) return
      stop()
      error.value = 'Audio playback failed.'
    }
    try {
      await learner.play()
      if (playing === learner && durationMs !== null && durationMs !== undefined)
        comparisonTimer = window.setTimeout(advance, durationMs + 1000)
    } catch (cause) {
      if (playing !== learner) return
      stop()
      error.value = `Audio playback failed: ${String(cause)}`
    }
  }

  function compare(recording: Recording, referenceUrl?: string) {
    stop()
    return compareUrl(recording.url, referenceUrl, recording.duration_ms)
  }

  function compareTake(take: StoredTake, referenceUrl: string) {
    stop()
    localUrl = URL.createObjectURL(new Blob([take.audio], { type: take.mime_type }))
    return compareUrl(localUrl, referenceUrl, take.duration_ms)
  }

  function background() {
    if (document.hidden) stop()
  }

  document.addEventListener('visibilitychange', background)
  onUnmounted(() => {
    document.removeEventListener('visibilitychange', background)
    stop()
  })
  return { play, compare, compareTake, stop, active, error }
}
