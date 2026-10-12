import { onMounted, onUnmounted, ref } from 'vue'
import type { Phrase } from '../api'

export function useReferenceAudio(phrase: () => Phrase | null, failure: () => string) {
  const playing = ref(false)
  const audioError = ref('')
  let audio: HTMLAudioElement | null = null
  function stopAudio() {
    if (audio) {
      audio.pause()
      audio.currentTime = 0
      audio.src = ''
      audio = null
    }
    playing.value = false
  }
  async function playReference() {
    if (playing.value) {
      stopAudio()
      return
    }
    const mediaId = phrase()?.content.reference_audio_id
    if (!mediaId) return
    audioError.value = ''
    const playback = new Audio(`/api/v1/media/${mediaId}`)
    audio = playback
    playback.onended = () => {
      if (audio === playback) stopAudio()
    }
    playback.onerror = () => {
      if (audio === playback) {
        stopAudio()
        audioError.value = failure()
      }
    }
    try {
      playing.value = true
      await playback.play()
    } catch {
      if (audio === playback) {
        stopAudio()
        audioError.value = failure()
      }
    }
  }

  function background() {
    if (document.hidden) stopAudio()
  }
  onMounted(() => document.addEventListener('visibilitychange', background))
  onUnmounted(() => {
    stopAudio()
    document.removeEventListener('visibilitychange', background)
  })
  return { playing, audioError, stopAudio, playReference }
}
