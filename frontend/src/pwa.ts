import { ref } from 'vue'
import { journalEntries } from './practiceJournal'
import { takeTransfers } from './takeTransfers'
import { takeEntries } from './takeJournalRecovery'
export const updateWaiting = ref(false)
let registration: ServiceWorkerRegistration | null = null
export async function verifyOfflineShell(repair = false): Promise<void> {
  const worker = navigator.serviceWorker?.controller
  if (!worker) throw new Error('Offline app installation is not ready. Retry after it finishes.')
  await new Promise<void>((resolve, reject) => {
    const channel = new MessageChannel()
    const timer = setTimeout(() => {
      channel.port1.close()
      reject(new Error('Offline app shell could not be verified. Update or reconnect and retry.'))
    }, 5000)
    channel.port1.onmessage = (event) => {
      clearTimeout(timer)
      channel.port1.close()
      if (event.data === true) resolve()
      else
        reject(
          new Error(
            'Offline app shell is unavailable or evicted. Reconnect and reinstall the app cache.',
          ),
        )
    }
    worker.postMessage({ type: repair ? 'repair-shell' : 'verify-shell' }, [channel.port2])
  })
}
export async function registerApp() {
  if (!('serviceWorker' in navigator)) return
  try {
    registration = await navigator.serviceWorker.register('/sw.js')
    const observe = () => {
      updateWaiting.value = Boolean(registration?.waiting && navigator.serviceWorker.controller)
    }
    observe()
    registration.addEventListener('updatefound', () =>
      registration?.installing?.addEventListener('statechange', observe),
    )
  } catch {
    /* Offline navigation can still use the previously installed worker. */
  }
}
export async function applyUpdate() {
  if (!registration?.waiting) return
  if (
    takeTransfers.hasVolatileTakes() ||
    (await journalEntries('event:')).length ||
    (await takeEntries()).length
  )
    return
  navigator.serviceWorker.addEventListener('controllerchange', () => location.reload(), {
    once: true,
  })
  registration.waiting.postMessage({ type: 'activate' })
}
