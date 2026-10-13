import { computed, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { usePracticeEvents } from './usePracticeEvents'
import { api, ApiError } from '../api'
import type { Language } from '../api'
import { journalDelete, journalEntries, journalPut } from '../practiceJournal'
import type { PendingEvent, PracticeSession, SessionStart } from '../practiceTypes'

export function usePracticeSession() {
  const session = ref<PracticeSession | null>(null)
  const busy = ref(false)
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
  const { pending, card, action, flush } = usePracticeEvents(session, error, accept)
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
      await journalPut(`session:${session.value.id}`, session.value)
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
  async function transition(status: 'active' | 'paused', state = session.value) {
    if (!state || busy.value) return
    busy.value = true
    try {
      await flush()
      const updated = await api<PracticeSession>(`/sessions/${state.id}`, {
        method: 'PATCH',
        body: JSON.stringify({ expected_version: state.version, status }),
      })
      accept(updated)
      await journalPut(`session:${updated.id}`, updated)
    } catch (cause) {
      error.value = String(cause)
    } finally {
      busy.value = false
    }
  }
  async function load() {
    const version = ++loadingVersion
    const id = routeId.value
    if (id !== session.value?.id) session.value = null
    try {
      const starts = await journalEntries<SessionStart>('start')
      candidate.value = starts[0]?.value ?? null
      pending.value = (await journalEntries<PendingEvent>('event:')).map((e) => e.value)
      const current = await api<PracticeSession | null>('/sessions/current')
      const selected = id ? await api<PracticeSession>(`/sessions/${id}`) : current
      if (version !== loadingVersion) return
      openSession.value = current
      session.value = selected
      if (selected) await journalPut(`session:${selected.id}`, selected)
      await flush()
    } catch (cause) {
      if (version !== loadingVersion) return
      error.value = String(cause)
      const cached = await journalEntries<PracticeSession>('session:').catch(() => [])
      const candidates = cached
        .map((e) => e.value)
        .filter((s) => (id ? s.id === id : s.status === 'active' || s.status === 'paused'))
      if (!session.value)
        session.value =
          candidates.sort((a, b) => b.updated_at.localeCompare(a.updated_at))[0] ?? null
    }
  }
  watch(routeId, load, { immediate: true })
  return {
    session,
    openSession,
    routeId,
    item,
    card,
    busy,
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
