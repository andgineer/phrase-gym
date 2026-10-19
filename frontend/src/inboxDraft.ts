import type { components } from '../tests/rust-contract/api.generated'

type Schemas = components['schemas']
export type InboxDraft = {
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
export async function inboxDraft(value?: InboxDraft | null): Promise<InboxDraft | undefined> {
  const db = await database()
  try {
    return await new Promise((resolve, reject) => {
      const tx = db.transaction('draft', value === undefined ? 'readonly' : 'readwrite')
      const store = tx.objectStore('draft')
      const request =
        value === undefined
          ? store.get('current')
          : value === null
            ? store.delete('current')
            : store.put(value, 'current')
      tx.oncomplete = () => resolve(value === undefined ? request.result : (value ?? undefined))
      tx.onabort = tx.onerror = () => reject(tx.error ?? new Error('Inbox storage failed'))
    })
  } finally {
    db.close()
  }
}

export async function audioInput(
  file: File,
): Promise<{ mime: Schemas['ReferenceMime']; base64: string }> {
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
  return { mime, base64: btoa(binary) }
}
