import { onUnmounted, ref } from 'vue'
import { setAudioSession } from '../audioSession'
import type { StoredTake } from '../practiceTypes'
import type { Recording } from '../studyTypes'

type Clip = { url: string; durationMs?: number | null }

export function usePlayback() {
  let playing: HTMLAudioElement | null = null
  const active = ref(false)
  const error = ref('')
  let localUrl: string | null = null
  let comparisonTimer: number | null = null

  function clearTimer() {
    if (comparisonTimer !== null) window.clearTimeout(comparisonTimer)
    comparisonTimer = null
  }

  function stop() {
    clearTimer()
    const current = playing
    playing = null
    if (current) {
      current.pause()
      current.src = ''
    }
    active.value = false
    if (localUrl) URL.revokeObjectURL(localUrl)
    localUrl = null
    setAudioSession('auto')
  }

  async function playClips([clip, ...remaining]: Clip[]) {
    if (!clip) return stop()
    error.value = ''
    setAudioSession('playback')
    const audio = new Audio(clip.url)
    playing = audio
    active.value = true
    const advance = () => {
      if (playing !== audio) return
      clearTimer()
      playing = null
      audio.pause()
      audio.src = ''
      void playClips(remaining)
    }
    audio.onended = advance
    audio.onerror = () => {
      if (playing !== audio) return
      stop()
      error.value = 'Audio playback failed.'
    }
    audio.onpause = () => {
      if (playing === audio && !audio.ended) stop()
    }
    try {
      await audio.play()
      // Some browser WebM encoders omit duration metadata and never signal the final frame.
      if (playing === audio && clip.durationMs !== null && clip.durationMs !== undefined)
        comparisonTimer = window.setTimeout(advance, clip.durationMs + 1000)
    } catch (cause) {
      if (playing !== audio) return
      stop()
      error.value = `Audio playback failed: ${String(cause)}`
    }
  }

  function play(url: string) {
    stop()
    return playClips([{ url }])
  }

  function compareUrl(url: string, referenceUrl?: string, durationMs?: number | null) {
    const learner = { url, durationMs }
    return playClips(referenceUrl ? [{ url: referenceUrl }, learner] : [learner])
  }

  function compare(recording: Recording, referenceUrl?: string) {
    stop()
    return compareUrl(recording.url, referenceUrl, recording.duration_ms)
  }

  function compareTake(take: StoredTake, referenceUrl?: string) {
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
