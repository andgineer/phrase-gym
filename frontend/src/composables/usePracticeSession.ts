import { computed, ref, watch } from 'vue'
import { takeTransfers } from '../takeTransfers'
import type { StoredTake } from '../practiceTypes'
import type { Outcome, Recording } from '../studyTypes'
import { useCapture } from './useCapture'
import { usePlayback } from './usePlayback'
import { usePractice } from './usePractice'

export function usePracticeSession() {
  const practice = usePractice()
  const playback = usePlayback()
  const card = computed(() => practice.view.value.card)
  // Takes of the open appearance stay playable here before their background upload finishes.
  const localTakes = ref<StoredTake[]>([])
  let mediaScope = 0
  let compareOnStop: number | null = null

  watch(
    () => card.value?.view_token,
    () => (localTakes.value = []),
  )

  const capture = useCapture({
    context: async () => {
      const current = card.value
      if (!current) throw new Error('No practice card is open')
      const { phrase_id, view_token, mode, kind, cue, hint } = current
      // IndexedDB cannot clone the card's reactive cue proxy.
      return { phrase_id, view_token, mode, kind, cue: cue ? { ...cue } : null, hint }
    },
    retain: takeTransfers.retain,
    finished: async (take: StoredTake) => {
      const current = card.value
      const sameView = current?.view_token === take.view_token
      if (sameView) localTakes.value = [...localTakes.value, take]
      const shouldCompare =
        take.complete && compareOnStop === mediaScope && sameView && !document.hidden
      compareOnStop = null
      // The hidden-answer front replays only the learner's take, never the reference.
      if (shouldCompare) void playback.compareTake(take, current?.answer?.reference_url)
      void takeTransfers.flush()
    },
    failure: async () => {},
  })

  const latestTake = computed<StoredTake | Recording | null>(
    () => localTakes.value.at(-1) ?? card.value?.recordings.at(-1) ?? null,
  )

  function playMine() {
    const take = latestTake.value
    if (!take) return
    if ('audio' in take) return playback.compareTake(take)
    return playback.play(take.url)
  }

  function compareMine() {
    const take = latestTake.value
    const reference = card.value?.answer?.reference_url
    if (!take || !reference) return
    if ('audio' in take) return playback.compareTake(take, reference)
    return playback.compare(take, reference)
  }

  async function settleMedia() {
    mediaScope++
    compareOnStop = null
    if (capture.capturing.value) await capture.stop()
    playback.stop()
  }

  async function complete(outcome: Outcome) {
    if (practice.busy.value) return
    await settleMedia()
    await practice.complete(outcome)
  }

  async function leave() {
    await settleMedia()
    await capture.release()
  }

  async function controlAudio() {
    if (capture.requesting.value) return capture.release()
    if (capture.capturing.value) {
      compareOnStop = mediaScope
      return capture.stop()
    }
    if (playback.active.value) return playback.stop()
    mediaScope++
    compareOnStop = null
    playback.stop()
    return capture.start()
  }

  return {
    practice,
    playback,
    capture,
    latestTake,
    playMine,
    compareMine,
    complete,
    leave,
    settleMedia,
    controlAudio,
  }
}
