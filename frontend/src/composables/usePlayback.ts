import { onUnmounted } from 'vue'
import { setAudioSession } from '../audioSession'
import type { Recording } from '../studyTypes'

export function usePlayback() {
  let playing: HTMLAudioElement | null = null

  function stop() {
    if (playing) {
      const current = playing
      playing = null
      current.pause()
      current.src = ''
    }
    setAudioSession('auto')
  }

  async function play(url: string) {
    stop()
    setAudioSession('playback')
    const audio = new Audio(url)
    playing = audio
    audio.onended = audio.onerror = () => {
      if (playing === audio) playing = null
      setAudioSession('auto')
    }
    await audio.play()
  }

  async function compare(recording: Recording, referenceUrl?: string) {
    stop()
    const learner = new Audio(recording.url)
    playing = learner
    learner.onended = () => {
      if (playing !== learner || !referenceUrl) return
      void play(referenceUrl)
    }
    learner.onpause = () => {
      if (playing !== learner) return
      learner.src = ''
      playing = null
      setAudioSession('auto')
    }
    learner.onerror = stop
    await learner.play()
  }

  onUnmounted(stop)
  return { play, compare, stop }
}
