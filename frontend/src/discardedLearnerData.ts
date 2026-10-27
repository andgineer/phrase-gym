import { discardedDataResetMarker, journalGet, resetDiscardedJournalOnce } from './practiceJournal'

const inboxDatabase = 'phrase-gym.inbox.v1'
const exposurePrefix = 'phrase-gym.reference-exposure.v1.'
const lastLearningLanguage = 'phrase-gym.last-learning-language.v1'

function deleteInbox(): Promise<void> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.deleteDatabase(inboxDatabase)
    request.onsuccess = () => resolve()
    request.onerror = () => reject(request.error ?? new Error('Old inbox reset failed'))
    request.onblocked = () => reject(new Error('Close other Phrase Gym tabs to clear old data'))
  })
}

async function reset() {
  if (await journalGet(discardedDataResetMarker)) return
  await deleteInbox()
  const keys = Array.from({ length: localStorage.length }, (_, index) => localStorage.key(index))
  for (const key of keys) if (key?.startsWith(exposurePrefix)) localStorage.removeItem(key)
  localStorage.removeItem(lastLearningLanguage)
  await resetDiscardedJournalOnce()
}

export async function resetDiscardedLearnerDataOnce() {
  if (typeof navigator !== 'undefined' && navigator.locks)
    await navigator.locks.request('phrase-gym.discarded-data-reset', reset)
  else await reset()
}
