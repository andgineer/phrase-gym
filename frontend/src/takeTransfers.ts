import { computed, reactive, ref } from 'vue'
import { releaseCapture, withAvailableTake } from './captureOwners'
import { ApiError, api } from './api'
import { journalDelete, journalGet, journalPut } from './practiceJournal'
import { acknowledgeTake, recoverTake, takeEntries, takePrefix } from './takeJournalRecovery'
import type { StoredTake } from './practiceTypes'
import type { components } from './api.generated'

type Receipt = components['schemas']['RecordingReceipt']
type Upload = components['schemas']['RecordingUpload']
type RejectedTake = { status: 'rejected'; id: string; key: string; message: string }
type Transfer = { status: 'pending' | 'failed' | 'retrying' } | RejectedTake
const transfers = reactive(new Map<string, Transfer>())
const savedRevision = ref(0)
const requestedRetries = reactive(new Set<string>())
let sending: Promise<void> | null = null
let scanRequested = false

const rejected = computed(() =>
  [...transfers.values()].filter((take): take is RejectedTake => take.status === 'rejected'),
)
const pendingUploads = computed(() => transfers.size - rejected.value.length)
const failedCount = computed(
  () => [...transfers.values()].filter((take) => take.status === 'failed').length,
)
const retrying = computed(
  () =>
    requestedRetries.size > 0 || [...transfers.values()].some((take) => take.status === 'retrying'),
)

// Conflict, deleted phrase, oversized or invalid takes never save; access refusals can later.
const permanentRefusals = new Set([409, 410, 413, 422])

function permanentlyRefused(cause: unknown) {
  return cause instanceof ApiError && permanentRefusals.has(cause.status)
}

function base64(audio: ArrayBuffer) {
  let value = ''
  const bytes = new Uint8Array(audio)
  for (let offset = 0; offset < bytes.length; offset += 8192)
    value += String.fromCharCode(...bytes.subarray(offset, offset + 8192))
  return btoa(value)
}

async function retain(take: StoredTake) {
  await journalPut(`${takePrefix}${take.id}`, take)
  if (take.status === 'pending') {
    transfers.set(take.id, { status: 'pending' })
    releaseCapture(take.id)
  }
}

async function readPending() {
  const pending = (await takeEntries()).filter(({ value }) => value.status === 'pending')
  const ids = new Set(pending.map(({ value }) => value.id))
  for (const id of transfers.keys()) {
    if (ids.has(id)) continue
    transfers.delete(id)
    requestedRetries.delete(id)
  }
  for (const id of ids) if (!transfers.has(id)) transfers.set(id, { status: 'pending' })
  return pending
}

async function upload(key: string, id: string) {
  if (transfers.get(id)?.status === 'rejected') return
  try {
    await withAvailableTake(id, async () => {
      const snapshot = await journalGet<StoredTake>(key)
      if (!snapshot || snapshot.id !== id || snapshot.status !== 'pending') return
      if (transfers.get(id)?.status === 'failed') transfers.set(id, { status: 'retrying' })
      const body: Upload = {
        id: snapshot.id,
        phrase_id: snapshot.phrase_id,
        view_token: snapshot.view_token,
        mode: snapshot.mode,
        kind: snapshot.kind,
        cue: snapshot.cue,
        hint: snapshot.hint,
        captured_at: snapshot.captured_at,
        mime_type: snapshot.mime_type,
        duration_ms: snapshot.duration_ms,
        complete: snapshot.complete,
        capture_settings: snapshot.capture_settings,
        audio_base64: base64(snapshot.audio),
      }
      const receipt = await api<Receipt>('/recordings', {
        method: 'POST',
        body: JSON.stringify(body),
      })
      const digest = await crypto.subtle.digest('SHA-256', snapshot.audio)
      const checksum = Array.from(new Uint8Array(digest), (byte) =>
        byte.toString(16).padStart(2, '0'),
      ).join('')
      if (
        receipt.id !== snapshot.id ||
        receipt.byte_length !== snapshot.audio.byteLength ||
        receipt.sha256 !== checksum ||
        receipt.complete !== snapshot.complete
      )
        throw new Error('Recording acknowledgement did not match the retained take')
      await acknowledgeTake(key, snapshot)
      transfers.delete(id)
      savedRevision.value++
    })
  } catch (cause) {
    if (permanentlyRefused(cause))
      transfers.set(id, { status: 'rejected', id, key, message: (cause as ApiError).message })
    else transfers.set(id, { status: 'failed' })
  }
}

function flush() {
  // A take can finish after the current pass has read the journal.
  scanRequested = true
  sending ??= (async () => {
    const attempted = new Set<string>()
    try {
      while (scanRequested) {
        scanRequested = false
        for (const {
          key,
          value: { id },
        } of await readPending()) {
          // A rescan finds newly stopped takes without repeating earlier failed uploads.
          if (attempted.has(id) && !requestedRetries.has(id)) continue
          attempted.add(id)
          await upload(key, id)
          requestedRetries.delete(id)
        }
        await readPending()
      }
    } finally {
      requestedRetries.clear()
      sending = null
    }
  })()
  return sending
}

function retry() {
  if (retrying.value || !failedCount.value) return
  for (const [id, take] of transfers) if (take.status === 'failed') requestedRetries.add(id)
  return flush()
}

async function recover() {
  for (const { key, value } of await takeEntries())
    if (value.status === 'capturing') await recoverTake(key)
  await flush()
}

async function discard(id: string) {
  const take = transfers.get(id)
  if (take?.status !== 'rejected') return
  if (await withAvailableTake(id, () => journalDelete(take.key))) transfers.delete(id)
}

export const takeTransfers = {
  pendingUploads,
  savedRevision,
  failedCount,
  retrying,
  rejected,
  retain,
  flush,
  retry,
  recover,
  discard,
}
