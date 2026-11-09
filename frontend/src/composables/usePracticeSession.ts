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
  const completing = ref(false)
  let mediaScope = 0
  let compareOnStop: { scope: number; reference: boolean } | null = null
  // A learner who just listened with Ref before a take needs only their own take replayed.
  let referenceHeard = false

  // Any card change, including a reconcile after a conflict, cancels the old phrase's media.
  watch(
    () => card.value?.view_token,
    () => {
      mediaScope++
      compareOnStop = null
      referenceHeard = false
      playback.stop()
      localTakes.value = []
    },
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
      const comparison = compareOnStop
      const shouldCompare =
        take.complete && comparison?.scope === mediaScope && sameView && !document.hidden
      const scope = mediaScope
      compareOnStop = null
      void takeTransfers.flush()
      if (!shouldCompare) return
      if (!current.revealed) await practice.reveal()
      const revealed = card.value
      // Reveal can finish after navigation or cancellation; its old take must stay silent.
      if (
        scope === mediaScope &&
        revealed?.view_token === take.view_token &&
        revealed.answer &&
        !document.hidden
      )
        void playback.compareTake(
          take,
          comparison.reference ? revealed.answer.reference_url : undefined,
        )
    },
    failure: async () => {},
  })

  const latestTake = computed<StoredTake | Recording | null>(
    () => localTakes.value.at(-1) ?? card.value?.recordings.at(-1) ?? null,
  )

  function playReference() {
    const reference = card.value?.answer?.reference_url
    if (!reference) return
    referenceHeard = true
    return playback.play(reference)
  }

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
    // Also waits for a stop already in progress, so the final take is retained first.
    await capture.stop()
    playback.stop()
  }

  async function complete(outcome: Outcome) {
    const target = card.value?.view_token
    if (completing.value || practice.busy.value || !target) return
    completing.value = true
    try {
      await settleMedia()
      // A completion is bound to the card it was pressed on, never to one shown meanwhile.
      if (card.value?.view_token === target) await practice.complete(outcome)
    } finally {
      completing.value = false
    }
  }

  async function leave() {
    await settleMedia()
    await capture.release()
  }

  async function controlAudio() {
    if (capture.requesting.value) return capture.release()
    if (capture.capturing.value) {
      compareOnStop = { scope: mediaScope, reference: !referenceHeard }
      referenceHeard = false
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
    completing,
    latestTake,
    playReference,
    playMine,
    compareMine,
    complete,
    leave,
    settleMedia,
    controlAudio,
  }
}
