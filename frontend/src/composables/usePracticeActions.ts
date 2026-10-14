import { computed, ref, watch } from 'vue'
import { serverUnavailable, connectionRetrying } from '../api'
import type { usePracticeSession } from './usePracticeSession'
import { useCardAudio } from './useCardAudio'
import type { RecallRating } from '../practiceTypes'

export function usePracticeActions(engine: ReturnType<typeof usePracticeSession>) {
  const { session, item, card, error, present, action, flush, transition } = engine
  const ratingBusy = ref(false)
  const preparing = ref(false)
  let preparation = 0
  const intro = computed(() =>
    Boolean(session.value?.manifest.dialogue && !session.value.queue[0]?.introduced),
  )
  const currentStep = computed(() => session.value?.queue[session.value.cursor])
  let captureContext: { sessionId: string; step: number } | null = null
  const media = useCardAudio(
    async () => {
      const origin = session.value
      const target = item.value
      const attempt = origin?.attempts.find((a) => a.step === origin.cursor)
      if (!origin || !attempt || !target) throw new Error('Attempt is not ready')
      captureContext = { sessionId: origin.id, step: origin.cursor }
      const captureEvent = await action('respond', { modality: 'recorded' })
      return {
        target_id: target.phrase.id,
        revision: target.phrase.revision,
        attempt_id: attempt.id,
        capture_event_id: captureEvent.id,
      }
    },
    () => `/api/v1/media/${item.value?.phrase.content.reference_audio_id}`,
    async () => {
      if (
        session.value?.id === captureContext?.sessionId &&
        session.value?.cursor === captureContext?.step
      )
        await action('failure', { reason: 'Capture unavailable or interrupted' })
    },
    () => `${session.value?.id}:${session.value?.cursor}`,
    async () => {
      if (
        session.value?.id === captureContext?.sessionId &&
        session.value?.cursor === captureContext?.step
      )
        await action('capture_complete')
    },
  )
  const { capturing, requesting, processing, error: captureError } = media.capture
  const { playing, error: audioError } = media.audio
  const { pendingTakes, error: takeError } = media.takes
  const { latestUrl } = media
  const ready = computed(() =>
    session.value?.attempts.some((a) => a.step === session.value?.cursor),
  )
  async function prepare() {
    if (engine.restoring.value || serverUnavailable.value || connectionRetrying.value) return
    const token = ++preparation
    preparing.value = true
    try {
      if (
        captureContext &&
        (session.value?.id !== captureContext.sessionId ||
          session.value?.cursor !== captureContext.step)
      ) {
        captureContext = null
        await media.release()
      }
      if (session.value?.status !== 'active') {
        await media.release()
        return
      }
      if (!item.value) return
      if (intro.value) {
        try {
          await action('dialogue_exposure')
        } catch {}
        return
      }
      latestUrl.value = ''
      try {
        await present()
        await media.refresh(item.value.phrase.id)
      } catch {}
    } finally {
      if (token === preparation) preparing.value = false
    }
  }
  watch(
    [
      () => session.value?.id,
      () => session.value?.cursor,
      () => session.value?.status,
      intro,
      engine.restoring,
      serverUnavailable,
      connectionRetrying,
    ],
    prepare,
    {
      immediate: true,
    },
  )
  async function help(kind: 'answer' | 'explain' | 'reference') {
    const origin = session.value
    const reference = `/api/v1/media/${item.value?.phrase.content.reference_audio_id}`
    media.audio.stop()
    try {
      await action(kind)
      if (
        kind === 'reference' &&
        session.value?.id === origin?.id &&
        session.value?.cursor === origin?.cursor
      )
        void media.audio.play(reference)
    } catch {}
  }
  async function rate(rating: RecallRating) {
    if (ratingBusy.value) return
    ratingBusy.value = true
    media.audio.stop()
    try {
      await action('rate', { rating })
      void media.takes.flush()
    } catch {
    } finally {
      ratingBusy.value = false
    }
  }
  async function pause() {
    await media.release()
    await transition('paused')
  }
  async function retry() {
    try {
      await engine.load()
      await flush()
      await prepare()
      await media.takes.flush()
      if (audioError.value) void media.audio.retry()
      else if (captureError.value) void media.capture.start()
    } catch {}
  }
  async function beginDialogue() {
    if (preparing.value || ratingBusy.value) return
    ratingBusy.value = true
    try {
      await action('dialogue_exposure')
      await action('dialogue_begin')
    } catch {
    } finally {
      ratingBusy.value = false
    }
  }
  async function isolate() {
    try {
      await action('detour')
    } catch {}
  }
  return {
    session,
    item,
    card,
    error,
    transition,
    intro,
    currentStep,
    media,
    capturing,
    requesting,
    processing,
    captureError,
    playing,
    audioError,
    pendingTakes,
    takeError,
    latestUrl,
    ready,
    preparing,
    ratingBusy,
    help,
    rate,
    pause,
    retry,
    beginDialogue,
    isolate,
  }
}
