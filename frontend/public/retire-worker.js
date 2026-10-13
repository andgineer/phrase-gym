// Retire earlier Phrase Gym caches without reloading tabs or touching pending evidence.
self.addEventListener('install', () => self.skipWaiting())
self.addEventListener('activate', (event) => {
  event.waitUntil((async () => {
    for (const name of await caches.keys())
      if (name.startsWith('phrase-gym.shell.') || name === 'phrase-gym.selected-media.v1')
        await caches.delete(name)
    await self.clients.claim()
    await self.registration.unregister()
  })())
})
self.addEventListener('message', (event) => {
  if (['verify-shell', 'repair-shell'].includes(event.data?.type))
    event.ports[0]?.postMessage(false)
})
