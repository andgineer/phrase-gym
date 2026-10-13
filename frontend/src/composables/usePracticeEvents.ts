import { computed, ref } from 'vue'
import type { Ref } from 'vue'
import { api, ApiError } from '../api'
import { journalCheckpoint, journalDelete, journalEntries, journalPut } from '../practiceJournal'
import { sessionCard } from '../practiceState'
import { localAttemptId, projectEvent } from '../offlineProjection'
import { verifyPack } from '../sessionPack'
import { restoreDeviceStatus } from '../deviceSessionState'
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
  const offline = ref(false)
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
  async function project(state: PracticeSession, isCurrent = () => true) {
    if (state.status === 'superseded' || state.status === 'completed') {
      if (isCurrent()) accept(state)
      return state
    }
    let projected = state
    const events = pending.value
      .filter((e) => e.sessionId === state.id)
      .sort((a, b) => a.event.sequence - b.event.sequence)
    for (const { event } of events)
      projected = projectEvent(projected, event, await localAttemptId(state.id, event.step))
    projected = await restoreDeviceStatus(projected)
    if (isCurrent()) accept(projected)
    return projected
  }
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
                // Retain the original entry too if writing the conflict record fails.
                await journalPut(`rejected-${stale.key}`, { ...stale.value, error: String(cause) })
                await journalDelete(stale.key)
              }
              pending.value = pending.value.filter((e) => e.sessionId !== entry.value.sessionId)
              const current = await api<PracticeSession>(`/sessions/${entry.value.sessionId}`)
              accept(current)
              await journalCheckpoint(current)
              throw new Error(
                'Session changed. Your conflicting actions are retained locally; repeat your choice on the current card.',
              )
            }
            throw cause
          }
          const checkpoint = await journalCheckpoint(receipt.session)
          await journalDelete(entry.key)
          pending.value = pending.value.filter((e) => e.event.id !== entry.value.event.id)
          await project(checkpoint)
          offline.value = false
        }
      }
      window.dispatchEvent(new Event('phrase-gym:receipts'))
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
      const downloaded = Boolean(await verifyPack(state.id))
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
        defer_repairs: downloaded && kind === 'rate',
      }
      await journalPut(`event:${event.id}`, { sessionId: state.id, event })
      pending.value.push({ sessionId: state.id, event })
      try {
        await flush()
      } catch (cause) {
        if (
          !downloaded ||
          !(cause instanceof TypeError || (cause instanceof ApiError && cause.status >= 500)) ||
          !pending.value.some((e) => e.event.id === event.id)
        )
          throw cause
        offline.value = true
        accept(projectEvent(state, event, await localAttemptId(state.id, event.step)))
        error.value = ''
      }
      return event
    })
    actions = operation.then(() => {}).catch(() => {})
    return operation.catch((cause) => {
      error.value = String(cause)
      throw cause
    })
  }
  return { pending, card, flush, action, offline, project }
}
