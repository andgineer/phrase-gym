import type { components } from './api.generated'

type Schemas = components['schemas']
export type InboxDraft = {
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
