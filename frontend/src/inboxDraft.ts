import type { components } from '../tests/rust-contract/api.generated'

type Schemas = components['schemas']
export type InboxDraft = {
  audio_sha256: string
  upload: Schemas['MediaUpload']
  submission: Schemas['LessonSubmission']
  receipt?: Schemas['LessonReceipt']
}

async function database(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open('phrase-gym.inbox.v1', 1)
    request.onupgradeneeded = () => request.result.createObjectStore('draft')
    request.onsuccess = () => resolve(request.result)
    request.onerror = () => reject(request.error)
  })
}
export async function inboxDraft(
  value?: InboxDraft | null,
  expectedId?: string,
): Promise<InboxDraft | undefined> {
  const db = await database()
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction('draft', value === undefined ? 'readonly' : 'readwrite')
      const store = tx.objectStore('draft')
      const request = store.get('current')
      let current: InboxDraft | undefined
      request.onsuccess = () => {
        current = request.result
        if (value === undefined || current?.submission.submission_id !== expectedId) return
        if (value === null) {
          store.delete('current')
          current = undefined
        } else {
          // Concurrent retries may add a receipt or upload binding, but never erase one.
          current = current
            ? {
                ...current,
                submission: current.submission.phrases[0]!.audio_sha256
                  ? current.submission
                  : value.submission,
                receipt: current.receipt ?? value.receipt,
              }
            : value
          store.put(current, 'current')
        }
      }
      tx.oncomplete = () => resolve(current)
      tx.onabort = tx.onerror = () => reject(tx.error ?? new Error('Inbox storage failed'))
    })
  } finally {
    db.close()
  }
}

export async function audioInput(
  file: File,
): Promise<{ mime: Schemas['ReferenceMime']; base64: string; sha256: string }> {
  const formats: Record<string, Schemas['ReferenceMime']> = {
    wav: 'audio/wav',
    mp3: 'audio/mpeg',
    ogg: 'audio/ogg',
    m4a: 'audio/mp4',
    mp4: 'audio/mp4',
  }
  const mime = formats[file.name.split('.').pop()?.toLowerCase() ?? '']
  if (!mime || !file.size || file.size > 2 * 1024 * 1024) throw new Error('audio')
  const bytes = new Uint8Array(await file.arrayBuffer())
  let binary = ''
  for (const byte of bytes) binary += String.fromCharCode(byte)
  const hash = await crypto.subtle.digest('SHA-256', bytes)
  const sha256 = Array.from(new Uint8Array(hash), (b) => b.toString(16).padStart(2, '0')).join('')
  return { mime, base64: btoa(binary), sha256 }
}
