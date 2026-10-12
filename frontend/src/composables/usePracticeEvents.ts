import { computed, ref } from 'vue'
import type { Ref } from 'vue'
import { api, ApiError } from '../api'
import { journalDelete, journalEntries, journalPut } from '../practiceJournal'
import { sessionCard } from '../practiceState'
import type {
  PendingEvent,
  PracticeBatchReceipt,
  PracticeInput,
  PracticeSession,
} from '../practiceTypes'

export function usePracticeEvents(
  session: Ref<PracticeSession | null>,
  error: Ref<string>,
  accept: (state: PracticeSession) => void,
) {
  const pending = ref<PendingEvent[]>([])
  let flushing: Promise<void> | null = null
  let actions = Promise.resolve()
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
      while (true) {
        const entries = await journalEntries<PendingEvent>('event:')
        if (!entries.length) break
        entries.sort((a, b) => a.value.event.sequence - b.value.event.sequence)
        for (const entry of entries) {
          let receipt: PracticeBatchReceipt
          try {
            receipt = await api<PracticeBatchReceipt>(`/sessions/${entry.value.sessionId}/events`, {
              method: 'POST',
              body: JSON.stringify({ events: [entry.value.event] }),
            })
          } catch (cause) {
            if (cause instanceof ApiError && cause.status === 409) {
              const rejected = entries.filter(
                (e) =>
                  e.value.sessionId === entry.value.sessionId &&
                  e.value.event.sequence >= entry.value.event.sequence,
              )
              for (const stale of rejected) {
                await journalPut(`rejected-${stale.key}`, { ...stale.value, error: String(cause) })
                await journalDelete(stale.key)
              }
              pending.value = pending.value.filter((e) => e.sessionId !== entry.value.sessionId)
              const current = await api<PracticeSession>(`/sessions/${entry.value.sessionId}`)
              accept(current)
              await journalPut(`session:${current.id}`, current)
              throw new Error(
                'Session changed. Your conflicting actions are retained locally; repeat your choice on the current card.',
              )
            }
            throw cause
          }
          await journalPut(`session:${receipt.session.id}`, receipt.session)
          await journalDelete(entry.key)
          accept(receipt.session)
          pending.value = pending.value.filter((e) => e.event.id !== entry.value.event.id)
        }
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
    const origin = session.value
    const step = origin?.cursor
    const operation = actions.then(async () => {
      const state = session.value
      if (state?.id !== origin?.id || state?.cursor !== step)
        throw new Error('The card changed; repeat your choice on the current card')
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
  return { pending, card, flush, action }
}
