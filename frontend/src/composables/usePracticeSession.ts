import { computed, ref, shallowReactive, watch } from 'vue'
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
  const reviewIndex = ref<number | null>(null)
  const displayedCard = computed(() => {
    const index = reviewIndex.value
    return (index === null ? null : practice.reviewedCards.value[index]?.card) ?? card.value
  })
  // Keep just the latest local take per appearance while its review is available.
  const localTakes = shallowReactive(new Map<string, StoredTake>())
  const completing = ref(false)
  let mediaScope = 0
  let compareOnStop: { scope: number; reference: boolean } | null = null
  // A learner who has listened with Ref on this card needs only their own takes replayed.
  let referenceHeard = false

  watch(practice.reviewedCards, () => {
    reviewIndex.value = null
    localTakes.clear()
  })
  watch(
    () => card.value?.view_token,
    (token) => {
      reviewIndex.value = null
      if (!token) localTakes.clear()
    },
  )
  watch(
    () => displayedCard.value?.view_token,
    () => {
      mediaScope++
      compareOnStop = null
      referenceHeard = false
      playback.stop()
    },
    { flush: 'sync' },
  )

  const capture = useCapture({
    context: async () => {
      const current = displayedCard.value
      if (!current) throw new Error('No practice card is open')
      const { phrase_id, view_token, mode, kind, cue, hint } = current
      // IndexedDB cannot clone the card's reactive cue proxy.
      return { phrase_id, view_token, mode, kind, cue: cue ? { ...cue } : null, hint }
    },
    retain: takeTransfers.retain,
    finished: async (take: StoredTake) => {
      const current = displayedCard.value
      const sameView = current?.view_token === take.view_token
      if (sameView) localTakes.set(take.view_token, take)
      const comparison = compareOnStop
      const shouldCompare =
        take.complete && comparison?.scope === mediaScope && sameView && !document.hidden
      const scope = mediaScope
      compareOnStop = null
      void takeTransfers.flush()
      if (!shouldCompare) return
      if (!current.revealed && reviewIndex.value === null) await practice.reveal()
      const revealed = displayedCard.value
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

  const latestTake = computed<StoredTake | Recording | null>(() => {
    const current = displayedCard.value
    return current
      ? (localTakes.get(current.view_token) ?? current.recordings.at(-1) ?? null)
      : null
  })

  function playReference() {
    const reference = displayedCard.value?.answer?.reference_url
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
    const reference = displayedCard.value?.answer?.reference_url
    if (!take || !reference) return
    if ('audio' in take) return playback.compareTake(take, reference)
    return playback.compare(take, reference)
  }

  function browse(index: number | null) {
    if (
      completing.value ||
      practice.busy.value ||
      capture.capturing.value ||
      capture.requesting.value ||
      capture.processing.value
    )
      return
    capture.error.value = ''
    reviewIndex.value = index
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
    if (reviewIndex.value !== null || completing.value || practice.busy.value || !target) return
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
    reviewIndex,
    browse,
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
