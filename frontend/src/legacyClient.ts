import { journalDelete, journalEntries } from './practiceJournal'

export async function retireLegacyClient() {
  const registrations = (await navigator.serviceWorker?.getRegistrations().catch(() => [])) ?? []
  for (const registration of registrations) {
    const worker = registration.active ?? registration.waiting ?? registration.installing
    if (
      !worker ||
      new URL(worker.scriptURL).origin !== location.origin ||
      new URL(worker.scriptURL).pathname !== '/sw.js'
    )
      continue
    await registration.update().catch(() => {})
    await registration.unregister()
  }
  if ('caches' in window)
    for (const name of await caches.keys())
      if (name.startsWith('phrase-gym.shell.') || name === 'phrase-gym.selected-media.v1')
        await caches.delete(name)
  const entries = await journalEntries<unknown>('').catch(() => [])
  for (const { key } of entries)
    if (
      key.startsWith('pack:') ||
      key.startsWith('lifecycle:') ||
      key.startsWith('paused:') ||
      key === 'preferences'
    )
      await journalDelete(key)
}
