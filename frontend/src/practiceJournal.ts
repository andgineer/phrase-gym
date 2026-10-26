import { toRaw } from 'vue'

const databaseName = 'phrase-gym.practice.v1'
let opening: Promise<IDBDatabase> | null = null
function database(): Promise<IDBDatabase> {
  if (!opening)
    opening = new Promise((resolve, reject) => {
      const request = indexedDB.open(databaseName, 1)
      request.onupgradeneeded = () => request.result.createObjectStore('journal')
      request.onsuccess = () => {
        const db = request.result
        db.onclose = () => {
          opening = null
        }
        db.onversionchange = () => {
          db.close()
          opening = null
        }
        resolve(db)
      }
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
export async function journalGet<T>(key: string): Promise<T | undefined> {
  const db = await database()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction('journal')
    const request = transaction.objectStore('journal').get(key)
    request.onsuccess = () => resolve(request.result as T | undefined)
    request.onerror = () => reject(request.error)
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
export async function journalModify<T>(
  key: string,
  update: (current: T | undefined) => T | null | undefined,
): Promise<T | undefined> {
  const db = await database()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction('journal', 'readwrite')
    const journal = transaction.objectStore('journal')
    const read = journal.get(key)
    let retained: T | undefined
    read.onsuccess = () => {
      retained = read.result as T | undefined
      try {
        const result = update(retained)
        if (result === null) {
          journal.delete(key)
          retained = undefined
        } else if (result !== undefined) {
          journal.put(toRaw(result), key)
          retained = result
        }
      } catch (cause) {
        transaction.abort()
        reject(cause)
      }
    }
    transaction.oncomplete = () => resolve(retained)
    transaction.onabort = transaction.onerror = () =>
      reject(transaction.error ?? new Error('Conditional journal write failed'))
  })
}
export async function journalEntries<T>(prefix: string): Promise<Array<{ key: string; value: T }>> {
  const db = await database()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction('journal')
    const entries: Array<{ key: string; value: T }> = []
    // Reading event/checkpoint keys must not clone unrelated retained audio buffers.
    const cursor = transaction
      .objectStore('journal')
      .openCursor(IDBKeyRange.bound(prefix, `${prefix}\uffff`))
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
