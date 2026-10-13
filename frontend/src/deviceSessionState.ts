import { journalDelete, journalEntries, journalPut } from './practiceJournal'
import type { PracticeSession } from './practiceTypes'
export type LocalStatus = 'active' | 'paused'
export async function restoreDeviceStatus(state: PracticeSession): Promise<PracticeSession> {
  if (state.status === 'completed' || state.status === 'superseded') return state
  const desired = (await journalEntries<LocalStatus>(`lifecycle:${state.id}`))[0]?.value
  const legacyPause = !desired && (await journalEntries(`paused:${state.id}`)).length > 0
  return desired || legacyPause ? { ...state, status: desired ?? 'paused' } : state
}
export async function retainDeviceStatus(id: string, status: LocalStatus) {
  await journalPut(`lifecycle:${id}`, status)
}
export async function clearDeviceStatus(id: string) {
  await journalDelete(`lifecycle:${id}`)
  await journalDelete(`paused:${id}`)
}
