import { onMounted, ref } from 'vue'
import { liveCaptureOwners } from '../captureOwners'
import { api } from '../api'
import { journalDelete, journalEntries, journalPut } from '../practiceJournal'
import type { RecordingReceipt, StoredTake } from '../practiceTypes'

function base64(bytes: ArrayBuffer) {
  const values = new Uint8Array(bytes)
  let binary = ''
  for (let i = 0; i < values.length; i += 8192)
    binary += String.fromCharCode(...values.subarray(i, i + 8192))
  return btoa(binary)
}
export function useTakeJournal() {
  const pendingTakes = ref(0)
  const error = ref('')
  const volatile = new Map<string, StoredTake>()
  let sending: Promise<void> | null = null
  let rerun = false
  async function retain(take: StoredTake) {
    volatile.set(take.id, take)
    try {
      await journalPut(`take:${take.id}`, take)
      volatile.delete(take.id)
      pendingTakes.value = (await journalEntries('take:')).length
    } catch (cause) {
      error.value = String(cause)
      throw cause
    }
  }
  async function flush() {
    if (sending) {
      rerun = true
      return sending
    }
    sending = (async () => {
      error.value = ''
      for (const take of volatile.values()) await retain(take)
      const entries = await journalEntries<StoredTake>('take:')
      pendingTakes.value = entries.length
      for (const { key, value: take } of entries) {
        if (take.status === 'capturing') continue
        const { audio, status: _status, owner: _owner, ...metadata } = take
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
        await journalDelete(key)
        pendingTakes.value--
      }
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
      const entries = await journalEntries<StoredTake>('take:')
      const live = entries.some((e) => e.value.status === 'capturing')
        ? await liveCaptureOwners()
        : new Set<string>()
      for (const { key, value: take } of entries) {
        if (take.status === 'capturing' && take.owner && live.has(take.owner)) continue
        if (!take.audio.byteLength) {
          await journalDelete(key)
          continue
        }
        if (take.status === 'capturing')
          await journalPut(key, { ...take, status: 'pending', complete: false })
      }
      await flush()
    } catch (cause) {
      error.value = String(cause)
    }
  }
  onMounted(recover)
  return { pendingTakes, error, retain, flush }
}
