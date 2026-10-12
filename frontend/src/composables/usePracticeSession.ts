import { computed, onMounted, ref } from 'vue'
import { api, ApiError } from '../api'
import type { Language } from '../api'
import { journalDelete, journalEntries, journalPut } from '../practiceJournal'
import { sessionCard } from '../practiceState'
import type {
  PendingEvent,
  PracticeBatchReceipt,
  PracticeInput,
  PracticeSession,
  SessionStart,
} from '../practiceTypes'

export function usePracticeSession() {
  const session = ref<PracticeSession | null>(null)
  const busy = ref(false)
  const error = ref('')
  const pending = ref<PendingEvent[]>([])
  const candidate = ref<SessionStart | null>(null)
  let flushing: Promise<void> | null = null
  let actions = Promise.resolve()
  const item = computed(() => {
    const state = session.value
    return state?.manifest.targets[state.queue[state.cursor]?.target_index ?? -1] ?? null
  })
  const card = computed(() =>
    session.value
      ? sessionCard(
          session.value,
          pending.value.filter((e) => e.sessionId === session.value?.id).map((e) => e.event),
        )
      : null,
  )

  async function flush() {
    if (flushing) return flushing
    flushing = (async () => {
      error.value = ''
      const entries = await journalEntries<PendingEvent>('event:')
      entries.sort((a, b) => a.value.event.sequence - b.value.event.sequence)
      for (const entry of entries) {
        const receipt = await api<PracticeBatchReceipt>(
          `/sessions/${entry.value.sessionId}/events`,
          {
            method: 'POST',
            body: JSON.stringify({ events: [entry.value.event] }),
          },
        )
        await journalPut(`session:${receipt.session.id}`, receipt.session)
        await journalDelete(entry.key)
        if (session.value?.id === receipt.session.id) session.value = receipt.session
        pending.value = pending.value.filter((e) => e.event.id !== entry.value.event.id)
      }
    })()
      .catch((cause) => {
        error.value = String(cause)
        throw cause
      })
      .finally(() => {
        flushing = null
      })
    return flushing
  }
  function action(
    kind: PracticeInput['kind'],
    options: Partial<PracticeInput> = {},
  ): Promise<PracticeInput> {
    const operation = actions.then(async () => {
      const state = session.value
      if (!state || state.status !== 'active') throw new Error('Resume the session first')
      const sequence =
        Math.max(
          state.last_sequence,
          ...pending.value.filter((e) => e.sessionId === state.id).map((e) => e.event.sequence),
        ) + 1
      const event: PracticeInput = {
        id: crypto.randomUUID(),
        sequence,
        step: state.cursor,
        occurred_at: new Date().toISOString(),
        kind,
        modality: 'self_report',
        ...options,
      }
      await journalPut(`event:${event.id}`, { sessionId: state.id, event })
      pending.value.push({ sessionId: state.id, event })
      await flush()
      return event
    })
    actions = operation.then(() => {}).catch(() => {})
    return operation.catch((cause) => {
      error.value = String(cause)
      throw cause
    })
  }
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
      const payload = candidate.value ?? {
        id: crypto.randomUUID(),
        language,
        lesson_id: lessonId || null,
        dialogue_id: dialogueId || null,
        takeover_session_id: takeover ? session.value?.id : null,
      }
      candidate.value = payload
      await journalPut('start', payload)
      session.value = await api<PracticeSession>('/sessions', {
        method: 'POST',
        body: JSON.stringify(payload),
      })
      await journalPut(`session:${session.value.id}`, session.value)
      await journalDelete('start')
      candidate.value = null
    } catch (cause) {
      error.value = String(cause)
      if (cause instanceof ApiError && cause.status < 500) {
        candidate.value = null
        await journalDelete('start')
        session.value = await api<PracticeSession | null>('/sessions/current').catch(
          () => session.value,
        )
      }
    } finally {
      busy.value = false
    }
  }
  async function transition(status: 'active' | 'paused') {
    const state = session.value
    if (!state || busy.value) return
    busy.value = true
    try {
      await flush()
      session.value = await api<PracticeSession>(`/sessions/${state.id}`, {
        method: 'PATCH',
        body: JSON.stringify({ expected_version: session.value?.version, status }),
      })
      if (session.value) await journalPut(`session:${session.value.id}`, session.value)
    } catch (cause) {
      error.value = String(cause)
    } finally {
      busy.value = false
    }
  }
  async function load() {
    try {
      const starts = await journalEntries<SessionStart>('start')
      candidate.value = starts[0]?.value ?? null
      pending.value = (await journalEntries<PendingEvent>('event:')).map((e) => e.value)
      const path = window.location.pathname.split('/').filter(Boolean)
      session.value = await api<PracticeSession | null>(
        path[2] === 'practice' && path[3] ? `/sessions/${path[3]}` : '/sessions/current',
      )
      if (session.value) await journalPut(`session:${session.value.id}`, session.value)
      await flush()
    } catch (cause) {
      error.value = String(cause)
      const cached = await journalEntries<PracticeSession>('session:').catch(() => [])
      const id = window.location.pathname.split('/').filter(Boolean)[3]
      const candidates = cached
        .map((e) => e.value)
        .filter((s) => (id ? s.id === id : s.status === 'active' || s.status === 'paused'))
      if (!session.value)
        session.value =
          candidates.sort((a, b) => b.updated_at.localeCompare(a.updated_at))[0] ?? null
    }
  }
  onMounted(load)
  return {
    session,
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
