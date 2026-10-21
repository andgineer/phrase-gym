import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { usePracticeEvents } from './usePracticeEvents'
import { api, ApiError, requireServer, serverUnavailable, connectionRetrying } from '../api'
import type { Language } from '../api'
import { journalCheckpoint, journalDelete, journalEntries, journalPut } from '../practiceJournal'
import type { PendingEvent, PracticeSession, SessionStart } from '../practiceTypes'
import { useSessionLifecycle } from './useSessionLifecycle'

export function usePracticeSession() {
  const session = ref<PracticeSession | null>(null)
  const busy = ref(false)
  const restoring = ref(false)
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
  const { pending, card, action, flush } = usePracticeEvents(
    session,
    error,
    accept,
    () =>
      !restoring.value &&
      !serverUnavailable.value &&
      !connectionRetrying.value &&
      !pending.value.length,
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
    voluntary = false,
  ) {
    if (busy.value) return
    requireServer()
    // Explicit start owns the next selection; an older library load must not replace it.
    loadingVersion += 1
    restoring.value = false
    busy.value = true
    error.value = ''
    try {
      await flush()
      const payload = candidate.value ?? {
        id: crypto.randomUUID(),
        voluntary,
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
  const lifecycle = useSessionLifecycle({ session, busy, error, flush, accept })
  const transition = lifecycle.transition
  async function load(): Promise<boolean> {
    const version = ++loadingVersion
    restoring.value = true
    const id = routeId.value
    if (id !== session.value?.id) session.value = null
    try {
      candidate.value = (await journalEntries<SessionStart>('start'))[0]?.value ?? null
      const events = await journalEntries<PendingEvent>('event:')
      if (version !== loadingVersion) return false
      pending.value = events.map((e) => e.value)
      try {
        await flush()
      } catch (cause) {
        if (pending.value.length) throw cause
      }
      if (version !== loadingVersion) return false
      await lifecycle.recover()
      if (candidate.value && !busy.value) {
        const created = await api<PracticeSession>('/sessions', {
          method: 'POST',
          body: JSON.stringify(candidate.value),
        })
        await journalCheckpoint(created)
        await journalDelete('start')
        candidate.value = null
      }
      const current = await api<PracticeSession | null>('/sessions/current')
      const selected = id ? await api<PracticeSession>(`/sessions/${id}`) : current
      const retainedCurrent = current ? await journalCheckpoint(current) : null
      const retainedSelected = selected
        ? selected.id === retainedCurrent?.id
          ? retainedCurrent
          : await journalCheckpoint(selected)
        : null
      if (version !== loadingVersion) return false
      openSession.value =
        retainedCurrent?.status === 'active' || retainedCurrent?.status === 'paused'
          ? retainedCurrent
          : null
      session.value = retainedSelected
      return true
    } catch (cause) {
      if (version === loadingVersion) error.value = String(cause)
      return false
    } finally {
      if (version === loadingVersion) restoring.value = false
    }
  }
  function reconcileLibrary() {
    if (!routeId.value && !restoring.value && !busy.value && document.visibilityState === 'visible')
      void load()
  }
  onMounted(() => {
    window.addEventListener('focus', reconcileLibrary)
    document.addEventListener('visibilitychange', reconcileLibrary)
  })
  onUnmounted(() => {
    window.removeEventListener('focus', reconcileLibrary)
    document.removeEventListener('visibilitychange', reconcileLibrary)
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
    error,
    pending,
    candidate,
    start,
    action,
    present,
    flush,
    transition,
    load,
  }
}
