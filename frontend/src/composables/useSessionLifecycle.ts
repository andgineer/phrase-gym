import type { Ref } from 'vue'
import { api, ApiError, requireServer } from '../api'
import { journalCheckpoint, journalDelete, journalEntries, journalPut } from '../practiceJournal'
import type { PracticeSession } from '../practiceTypes'

type Transition = { sessionId: string; expected_version: number; status: 'active' | 'paused' }
type Host = {
  session: Ref<PracticeSession | null>
  busy: Ref<boolean>
  error: Ref<string>
  flush: () => Promise<void>
  accept: (state: PracticeSession) => void
}
export function useSessionLifecycle(host: Host) {
  async function send(key: string, value: Transition) {
    const updated = await api<PracticeSession>(`/sessions/${value.sessionId}`, {
      method: 'PATCH',
      body: JSON.stringify({ expected_version: value.expected_version, status: value.status }),
    })
    const checkpoint = await journalCheckpoint(updated)
    await journalDelete(key)
    host.accept(checkpoint)
  }
  async function recover() {
    for (const { key, value } of await journalEntries<Transition>('transition:')) {
      try {
        await send(key, value)
      } catch (cause) {
        if (!(cause instanceof ApiError && cause.status === 409)) throw cause
        await journalPut(`rejected-${key}`, { ...value, error: String(cause) })
        await journalDelete(key)
      }
    }
  }
  async function transition(status: Transition['status'], state = host.session.value) {
    if (!state || host.busy.value) return
    requireServer()
    host.busy.value = true
    host.error.value = ''
    try {
      await host.flush()
      const checkpoint =
        (await journalEntries<PracticeSession>(`session:${state.id}`))[0]?.value ?? state
      const value = { sessionId: state.id, expected_version: checkpoint.version, status }
      const key = `transition:${state.id}`
      await journalPut(key, value)
      await send(key, value)
    } catch (cause) {
      host.error.value = String(cause)
    } finally {
      host.busy.value = false
    }
  }
  return { transition, recover }
}
