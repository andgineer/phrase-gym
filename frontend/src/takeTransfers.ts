import { computed, ref } from 'vue'
import { releaseCapture, withAvailableTake } from './captureOwners'
import { ApiError, api } from './api'
import { journalDelete, journalGet, journalPut } from './practiceJournal'
import { acknowledgeTake, recoverTake, takeEntries, takePrefix } from './takeJournalRecovery'
import type { StoredTake } from './practiceTypes'
import type { components } from './api.generated'

type Receipt = components['schemas']['RecordingReceipt']
type Upload = components['schemas']['RecordingUpload']
type RejectedTake = { id: string; key: string; message: string }
const pendingTakes = ref(0)
const error = ref('')
const rejected = ref<RejectedTake[]>([])
let sending: Promise<void> | null = null
// Refused takes stay stored for an explicit discard but will never upload.
const uploadingTakes = computed(() => Math.max(0, pendingTakes.value - rejected.value.length))

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
  if (take.status === 'pending') releaseCapture(take.id)
  pendingTakes.value = (await takeEntries()).length
}

async function flush() {
  if (sending) return sending
  sending = (async () => {
    error.value = ''
    const refused: RejectedTake[] = []
    for (const { key, value: listed } of await takeEntries()) {
      if (listed.status !== 'pending') continue
      try {
        await withAvailableTake(listed.id, async () => {
          const snapshot = await journalGet<StoredTake>(key)
          if (!snapshot || snapshot.id !== listed.id || snapshot.status !== 'pending') return
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
        })
      } catch (cause) {
        if (permanentlyRefused(cause))
          refused.push({ id: listed.id, key, message: (cause as ApiError).message })
        else error.value = `${String(cause)} Audio remains on this device.`
      }
    }
    rejected.value = refused
    pendingTakes.value = (await takeEntries()).length
  })().finally(() => (sending = null))
  return sending
}

async function recover() {
  for (const { key, value } of await takeEntries())
    if (value.status === 'capturing') await recoverTake(key)
  await flush()
}

async function discard(id: string) {
  const take = rejected.value.find((candidate) => candidate.id === id)
  if (!take) return
  await withAvailableTake(id, () => journalDelete(take.key))
  rejected.value = rejected.value.filter((candidate) => candidate.id !== id)
  pendingTakes.value = (await takeEntries()).length
}

export const takeTransfers = {
  pendingTakes,
  uploadingTakes,
  error,
  rejected,
  retain,
  flush,
  recover,
  discard,
}
