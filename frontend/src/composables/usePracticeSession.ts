import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { usePracticeEvents } from './usePracticeEvents'
import { api, ApiError } from '../api'
import type { Language } from '../api'
import { journalCheckpoint, journalDelete, journalEntries, journalPut } from '../practiceJournal'
import type { PendingEvent, PracticeSession, SessionStart } from '../practiceTypes'
import { verifyPack } from '../sessionPack'
import { restoreDeviceStatus } from '../deviceSessionState'
import { useSessionLifecycle } from './useSessionLifecycle'

export function usePracticeSession() {
  const session = ref<PracticeSession | null>(null)
  const busy = ref(false)
  const restoring = ref(false)
  const offlineUnavailable = ref(false)
  const error = ref('')
  const route = useRoute()
  const openSession = ref<PracticeSession | null>(null)
  const candidate = ref<SessionStart | null>(null)
  const routeId = computed(() =>
    route.path.split('/').filter(Boolean)[2] === 'practice'
      ? route.path.split('/').filter(Boolean)[3]
      : null,
  )
  let loadingVersion = 0
  function accept(state: PracticeSession) {
    if (state.status === 'active' || state.status === 'paused') openSession.value = state
    else if (openSession.value?.id === state.id) openSession.value = null
    if (routeId.value ? routeId.value === state.id : session.value?.id === state.id)
      session.value = state
  }
  const { pending, card, action, flush, offline, project } = usePracticeEvents(
    session,
    error,
    accept,
    () => !restoring.value && !offlineUnavailable.value,
  )
  const item = computed(() => {
    const state = session.value
    return state?.manifest.targets[state.queue[state.cursor]?.target_index ?? -1] ?? null
  })
  async function present() {
    const state = session.value
    if (
      state?.status === 'active' &&
      item.value &&
      !state.attempts.some((a) => a.step === state.cursor) &&
      !pending.value.some(
        (e) =>
          e.sessionId === state.id && e.event.step === state.cursor && e.event.kind === 'present',
      )
    ) {
      await action('present')
    }
  }
  async function start(
    language: Language,
    lessonId?: string,
    dialogueId?: string,
    takeover = false,
  ) {
    if (busy.value) return
    busy.value = true
    error.value = ''
    try {
      await flush()
      const payload = candidate.value ?? {
        id: crypto.randomUUID(),
        language,
        lesson_id: lessonId || null,
        dialogue_id: dialogueId || null,
        takeover_session_id: takeover ? openSession.value?.id : null,
      }
      candidate.value = payload
      await journalPut('start', payload)
      session.value = await api<PracticeSession>('/sessions', {
        method: 'POST',
        body: JSON.stringify(payload),
      })
      openSession.value = session.value
      await journalCheckpoint(session.value)
      await journalDelete('start')
      candidate.value = null
    } catch (cause) {
      error.value = String(cause)
      if (cause instanceof ApiError && cause.status < 500) {
        candidate.value = null
        await journalDelete('start')
        openSession.value = await api<PracticeSession | null>('/sessions/current').catch(
          () => openSession.value,
        )
      }
    } finally {
      busy.value = false
    }
  }
  const transition = useSessionLifecycle({ session, busy, error, offline, flush, accept })
  async function load() {
    const version = ++loadingVersion
    restoring.value = true
    offlineUnavailable.value = false
    const id = routeId.value
    if (id !== session.value?.id) session.value = null
    try {
      const starts = await journalEntries<SessionStart>('start')
      if (version !== loadingVersion) return
      candidate.value = starts[0]?.value ?? null
      const events = await journalEntries<PendingEvent>('event:')
      if (version !== loadingVersion) return
      pending.value = events.map((e) => e.value)
      const current = await api<PracticeSession | null>('/sessions/current')
      const selected = id ? await api<PracticeSession>(`/sessions/${id}`) : current
      if (version !== loadingVersion) return
      openSession.value = current
      const checkpoint = selected ? await journalCheckpoint(selected) : null
      if (version !== loadingVersion) return
      const restoredCheckpoint = checkpoint ? await restoreDeviceStatus(checkpoint) : null
      if (version !== loadingVersion) return
      const projected = restoredCheckpoint
        ? await project(restoredCheckpoint, () => version === loadingVersion)
        : null
      if (version !== loadingVersion) return
      session.value = projected
      await flush()
      if (version === loadingVersion) offline.value = false
    } catch (cause) {
      if (version !== loadingVersion) return
      error.value = String(cause)
      const cached = await journalEntries<PracticeSession>('session:').catch(() => [])
      if (version !== loadingVersion) return
      const candidates = cached
        .map((e) => e.value)
        .filter((s) => (id ? s.id === id : s.status === 'active' || s.status === 'paused'))
      const restored =
        [...candidates, ...(session.value ? [session.value] : [])].sort(
          (a, b) =>
            (a.id === b.id ? b.last_sequence - a.last_sequence || b.version - a.version : 0) ||
            b.updated_at.localeCompare(a.updated_at),
        )[0] ?? null
      if (restored && (!id || restored.id === id)) {
        try {
          const pack = await verifyPack(restored.id)
          if (version !== loadingVersion) return
          if (pack) {
            offline.value = true
            await project(restored, () => version === loadingVersion)
            if (version === loadingVersion) error.value = ''
          } else accept(restored)
        } catch (packError) {
          if (version !== loadingVersion) return
          offlineUnavailable.value = true
          accept(restored)
          error.value = String(packError)
        }
      } else if (id) {
        error.value =
          'Session data is unavailable on this device. Reconnect to restore server-acknowledged progress; unacknowledged events and audio cannot be restored after browser storage eviction.'
      }
    } finally {
      if (version === loadingVersion) restoring.value = false
    }
  }
  function reconnect() {
    if (!document.hidden) void load()
  }
  onMounted(() => {
    window.addEventListener('online', reconnect)
    document.addEventListener('visibilitychange', reconnect)
  })
  onUnmounted(() => {
    window.removeEventListener('online', reconnect)
    document.removeEventListener('visibilitychange', reconnect)
  })
  watch(routeId, load, { immediate: true })
  return {
    session,
    openSession,
    routeId,
    item,
    card,
    busy,
    restoring,
    offlineUnavailable,
    error,
    pending,
    offline,
    candidate,
    start,
    action,
    present,
    flush,
    transition,
    load,
  }
}
