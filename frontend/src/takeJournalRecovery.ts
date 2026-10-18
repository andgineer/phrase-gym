import { journalEntries, journalModify } from './practiceJournal'
import type { StoredTake } from './practiceTypes'

export const takePrefix = 'take-v2:'
export async function takeEntries() {
  return journalEntries<StoredTake>(takePrefix)
}

export async function recoverTake(key: string) {
  await journalModify<StoredTake>(key, (current) => {
    if (!current || current.status !== 'capturing') return
    return current.audio.byteLength ? { ...current, status: 'pending', complete: false } : null
  })
}
function matches(left: StoredTake, right: StoredTake): boolean {
  const a = left as unknown as Record<string, unknown>
  const b = right as unknown as Record<string, unknown>
  const keys = new Set([...Object.keys(a), ...Object.keys(b)])
  for (const key of keys)
    if (key !== 'audio' && JSON.stringify(a[key]) !== JSON.stringify(b[key])) return false
  if (left.audio.byteLength !== right.audio.byteLength) return false
  const leftBytes = new Uint8Array(left.audio)
  const rightBytes = new Uint8Array(right.audio)
  return leftBytes.every((value, index) => value === rightBytes[index])
}
export async function acknowledgeTake(key: string, uploaded: StoredTake) {
  const retained = await journalModify<StoredTake>(key, (current) =>
    current && matches(current, uploaded) ? null : undefined,
  )
  if (retained)
    throw new Error('Recording changed during transfer. Newer audio remains on this device.')
}
