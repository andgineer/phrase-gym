import { ref } from 'vue'
import { releaseCapture, withAvailableTake } from './captureOwners'
import { api } from './api'
import { journalGet, journalPut } from './practiceJournal'
import { acknowledgeTake, recoverTake, takeEntries, takePrefix } from './takeJournalRecovery'
import type { StoredTake } from './practiceTypes'
import type { components } from './api.generated'

type Receipt = components['schemas']['RecordingReceipt']
type Upload = components['schemas']['RecordingUpload']
const pendingTakes = ref(0)
const error = ref('')
let sending: Promise<void> | null = null

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
    for (const { key, value: listed } of await takeEntries()) {
      if (listed.status !== 'pending') continue
      try {
        await withAvailableTake(listed.id, async () => {
          const snapshot = await journalGet<StoredTake>(key)
          if (!snapshot || snapshot.id !== listed.id || snapshot.status !== 'pending') return
          const body: Upload = {
            id: snapshot.id,
            card_id: snapshot.card_id,
            attempt_id: snapshot.attempt_id,
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
        error.value = `${String(cause)} Audio remains on this device.`
      }
    }
    pendingTakes.value = (await takeEntries()).length
  })().finally(() => (sending = null))
  return sending
}

async function recover() {
  for (const { key, value } of await takeEntries())
    if (value.status === 'capturing') await recoverTake(key)
  await flush()
}

export const takeTransfers = { pendingTakes, error, retain, flush, recover }
