import { ref } from 'vue'
import { releaseCapture, withAvailableTake } from './captureOwners'
import { recoverTake, acknowledgeTake, takeEntries, takePrefix } from './takeJournalRecovery'
import { api, ApiError } from './api'
import { journalEntries, journalPut } from './practiceJournal'
import type { RecordingReceipt, StoredTake } from './practiceTypes'

function base64(bytes: ArrayBuffer) {
  const values = new Uint8Array(bytes)
  let binary = ''
  for (let i = 0; i < values.length; i += 8192)
    binary += String.fromCharCode(...values.subarray(i, i + 8192))
  return btoa(binary)
}
const pendingTakes = ref(0)
const error = ref('')
const volatile = new Map<string, StoredTake>()
let sending: Promise<void> | null = null
let rerun = false
let waitingForReceipts = false
async function retain(take: StoredTake) {
  volatile.set(take.id, take)
  try {
    await journalPut(`${takePrefix}${take.id}`, take)
    if (volatile.get(take.id) === take) volatile.delete(take.id)
    if (take.status === 'pending') releaseCapture(take.id)
    pendingTakes.value = (await localTakes()).length
  } catch (cause) {
    if (!take.audio.byteLength) volatile.delete(take.id)
    pendingTakes.value = (await localTakes()).length
    error.value = `Storage unavailable: ${String(cause)}. Audio is only in this open tab. Keep it open, free space and retry.`
    throw new Error(error.value)
  }
}
async function flush() {
  if (sending) {
    rerun = true
    return sending
  }
  sending = (async () => {
    error.value = ''
    const failures: string[] = []
    for (const take of volatile.values()) {
      if (take.status === 'capturing') continue
      try {
        await retain(take)
      } catch (cause) {
        failures.push(String(cause))
      }
    }
    const entries = await takeEntries()
    pendingTakes.value = (await localTakes()).length
    waitingForReceipts = false
    for (const { key, value: snapshot } of entries) {
      if (snapshot.status === 'capturing' || volatile.has(snapshot.id)) continue
      try {
        await withAvailableTake(snapshot.id, async () => {
          const take = (await journalEntries<StoredTake>(key)).find(
            (entry) => entry.key === key,
          )?.value
          if (!take || take.status !== 'pending' || volatile.has(take.id)) return
          const { audio, status: _status, owner: _owner, capture_lease: _lease, ...metadata } = take
          const receipt = await api<RecordingReceipt>('/recordings', {
            method: 'POST',
            body: JSON.stringify({ ...metadata, audio_base64: base64(audio) }),
          })
          const digest = await crypto.subtle.digest('SHA-256', audio)
          const checksum = Array.from(new Uint8Array(digest), (b) =>
            b.toString(16).padStart(2, '0'),
          ).join('')
          if (
            receipt.id !== take.id ||
            receipt.byte_length !== audio.byteLength ||
            receipt.sha256 !== checksum ||
            receipt.complete !== take.complete
          )
            throw new Error('Recording acknowledgement did not match the retained take')
          await acknowledgeTake(key, take)
          pendingTakes.value = (await localTakes()).length
        })
      } catch (cause) {
        if (cause instanceof ApiError && cause.status === 409) waitingForReceipts = true
        failures.push(`Recording ${snapshot.id}: ${String(cause)}. Audio remains on this device.`)
      }
    }
    error.value = failures.join('\n')
  })()
    .catch((cause) => {
      error.value = String(cause)
    })
    .finally(() => {
      sending = null
      if (rerun) {
        rerun = false
        void flush()
      }
    })
  return sending
}
async function recover() {
  try {
    const entries = await takeEntries()
    const failures: string[] = []
    for (const { key, value: take } of entries) {
      if (take.status !== 'capturing' || volatile.has(take.id)) continue
      try {
        await withAvailableTake(take.id, () => recoverTake(key))
      } catch (cause) {
        failures.push(String(cause))
      }
    }
    await flush()
    if (failures.length) error.value = [error.value, ...failures].filter(Boolean).join('\n')
  } catch (cause) {
    error.value = String(cause)
  }
}
async function localTakes(): Promise<StoredTake[]> {
  const stored = (await takeEntries().catch(() => [])).map((entry) => entry.value)
  const merged = new Map(stored.map((take) => [take.id, take]))
  for (const take of volatile.values()) merged.set(take.id, take)
  return [...merged.values()]
}
function hasVolatileTakes() {
  return volatile.size > 0
}
function retryAfterReceipts() {
  if (waitingForReceipts) void flush()
}
export const takeTransfers = {
  retryAfterReceipts,
  pendingTakes,
  error,
  retain,
  flush,
  recover,
  localTakes,
  hasVolatileTakes,
}
