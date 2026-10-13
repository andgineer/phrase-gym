/* SHELL_VERSION and SHELL_ASSETS are replaced by the production build. */
const shell = 'phrase-gym.shell.SHELL_VERSION'
const assets = SHELL_ASSETS
self.addEventListener('install', (event) => {
  event.waitUntil(caches.open(shell).then((cache) => cache.addAll(assets)))
})
self.addEventListener('activate', (event) => { event.waitUntil(self.clients.claim()) })
self.addEventListener('message', (event) => {
  if (event.data?.type === 'activate') self.skipWaiting()
  if (['verify-shell', 'repair-shell'].includes(event.data?.type)) event.waitUntil(caches.open(shell).then(async (cache) => {
    let responses = await Promise.all(assets.map((asset) => cache.match(asset)))
    if (!responses.every(Boolean) && event.data.type === 'repair-shell') {
      const index = await fetch('/index.html', { cache: 'no-store' })
      const digest = await crypto.subtle.digest('SHA-256', await index.clone().arrayBuffer())
      const version = Array.from(new Uint8Array(digest), (b) => b.toString(16).padStart(2, '0')).join('').slice(0, 16)
      if (!index.ok || !shell.endsWith(version)) throw new Error('A compatible app update is required')
      await cache.addAll(assets)
      responses = await Promise.all(assets.map((asset) => cache.match(asset)))
    }
    event.ports[0]?.postMessage(responses.every(Boolean))
  }).catch(() => event.ports[0]?.postMessage(false)))
})
self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url)
  if (url.origin !== self.location.origin || event.request.method !== 'GET') return
  if (/^\/api\/v1\/media\/[^/]+$/.test(url.pathname)) {
    event.respondWith(caches.open('phrase-gym.selected-media.v1').then(async (cache) => {
      const saved = await cache.match(url.pathname)
      if (!saved) return fetch(event.request)
      const range = event.request.headers.get('range')
      if (!range) return saved
      const bytes = await saved.arrayBuffer()
      const match = /^bytes=(\d*)-(\d*)$/.exec(range)
      if (!match) return new Response(null, { status: 416 })
      const start = match[1] ? Number(match[1]) : Math.max(0, bytes.byteLength - Number(match[2]))
      const end = match[1] && match[2] ? Math.min(Number(match[2]), bytes.byteLength - 1) : bytes.byteLength - 1
      if (start > end) return new Response(null, { status: 416 })
      return new Response(bytes.slice(start, end + 1), { status: 206, headers: {
        'Content-Type': saved.headers.get('Content-Type') || 'application/octet-stream',
        'Content-Range': `bytes ${start}-${end}/${bytes.byteLength}`, 'Content-Length': String(end - start + 1),
      } })
    }))
  } else if (event.request.mode === 'navigate') {
    event.respondWith(caches.open(shell).then((cache) => cache.match('/index.html')).then((saved) => saved || fetch(event.request)))
  } else if (assets.includes(url.pathname) || url.pathname.startsWith('/assets/')) {
    event.respondWith(caches.match(url.pathname).then((saved) => saved || fetch(event.request)))
  }
})
