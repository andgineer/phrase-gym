import type { Ref } from 'vue'
import { api, ApiError } from '../api'
import { journalCheckpoint, journalEntries } from '../practiceJournal'
import { clearDeviceStatus, retainDeviceStatus } from '../deviceSessionState'
import type { LocalStatus } from '../deviceSessionState'
import { verifyPack } from '../sessionPack'
import type { PracticeSession } from '../practiceTypes'

type Host = {
  session: Ref<PracticeSession | null>
  busy: Ref<boolean>
  error: Ref<string>
  offline: Ref<boolean>
  flush: () => Promise<void>
  accept: (state: PracticeSession) => void
}
export function useSessionLifecycle(host: Host) {
  async function locally(status: LocalStatus, state: PracticeSession) {
    if (!(await verifyPack(state.id)))
      throw new Error('Download this session before using it offline')
    await retainDeviceStatus(state.id, status)
    const current = host.session.value?.id === state.id ? host.session.value : state
    if (current.status !== 'completed' && current.status !== 'superseded')
      host.accept({ ...current, status })
    host.offline.value = true
    host.error.value = ''
  }
  async function transition(status: LocalStatus, state = host.session.value) {
    if (!state || host.busy.value) return
    host.busy.value = true
    host.error.value = ''
    try {
      if (host.offline.value) {
        await locally(status, state)
        return
      }
      try {
        await host.flush()
        const checkpoint =
          (await journalEntries<PracticeSession>(`session:${state.id}`))[0]?.value ?? state
        const updated = await api<PracticeSession>(`/sessions/${state.id}`, {
          method: 'PATCH',
          body: JSON.stringify({ expected_version: checkpoint.version, status }),
        })
        const retained = await journalCheckpoint(updated)
        await clearDeviceStatus(state.id)
        host.accept(retained)
      } catch (cause) {
        if (!(cause instanceof TypeError || (cause instanceof ApiError && cause.status >= 500)))
          throw cause
        await locally(status, state)
      }
    } catch (cause) {
      host.error.value = String(cause)
    } finally {
      host.busy.value = false
    }
  }
  return transition
}
