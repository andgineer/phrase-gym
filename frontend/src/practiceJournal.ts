import { toRaw } from 'vue'

const databaseName = 'phrase-gym.practice.v1'
let opening: Promise<IDBDatabase> | null = null
function database(): Promise<IDBDatabase> {
  if (!opening)
    opening = new Promise((resolve, reject) => {
      const request = indexedDB.open(databaseName, 1)
      request.onupgradeneeded = () => request.result.createObjectStore('journal')
      request.onsuccess = () => resolve(request.result)
      request.onerror = () => {
        opening = null
        reject(request.error)
      }
    })
  return opening
}

export async function journalPut(key: string, value: unknown): Promise<void> {
  const db = await database()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction('journal', 'readwrite')
    // IndexedDB cannot clone Vue's reactive proxies.
    transaction.objectStore('journal').put(toRaw(value), key)
    transaction.oncomplete = () => resolve()
    transaction.onabort = transaction.onerror = () =>
      reject(transaction.error ?? new Error('Journal write failed'))
  })
}
export async function journalDelete(key: string): Promise<void> {
  const db = await database()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction('journal', 'readwrite')
    transaction.objectStore('journal').delete(key)
    transaction.oncomplete = () => resolve()
    transaction.onabort = transaction.onerror = () =>
      reject(transaction.error ?? new Error('Journal delete failed'))
  })
}
export async function journalEntries<T>(prefix: string): Promise<Array<{ key: string; value: T }>> {
  const db = await database()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction('journal')
    const entries: Array<{ key: string; value: T }> = []
    const cursor = transaction.objectStore('journal').openCursor()
    cursor.onsuccess = () => {
      const next = cursor.result
      if (next) {
        if (String(next.key).startsWith(prefix))
          entries.push({ key: String(next.key), value: next.value as T })
        next.continue()
      }
    }
    transaction.oncomplete = () => resolve(entries)
    transaction.onabort = transaction.onerror = () => reject(transaction.error)
  })
}
