import { silencePlayback } from './silent-playback'
import { createServer, request } from 'node:http'
import type { ServerResponse } from 'node:http'
import type { Socket } from 'node:net'
import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test as base } from './fixtures'

type Connection = {
  url: string
  disconnect: () => void
  reconnect: () => void
  cutNextSessionBody: () => () => void
}
export const test = base.extend<{ connection: Connection; journalDiagnostics: void }>({
  journalDiagnostics: [
    async ({ page }, use, testInfo) => {
      await use()
      if (testInfo.status === testInfo.expectedStatus || page.isClosed()) return
      const state = await page
        .evaluate(async () => {
          const entries = await new Promise<unknown[]>((resolve) => {
            const opening = indexedDB.open('phrase-gym.practice.v1', 1)
            opening.onsuccess = () => {
              const request = opening.result
                .transaction('journal')
                .objectStore('journal')
                .openCursor()
              const rows: unknown[] = []
              request.onsuccess = () => {
                const cursor = request.result
                if (!cursor) {
                  resolve(rows)
                  return
                }
                if (/^(event:|session:|transition:)/.test(String(cursor.key)))
                  rows.push({ key: cursor.key, value: cursor.value })
                cursor.continue()
              }
            }
          })
          return { entries, text: document.querySelector('main')?.innerText }
        })
        .catch(() => null)
      await testInfo.attach('synthetic-journal-diagnostics', {
        body: Buffer.from(JSON.stringify(state, null, 2)),
        contentType: 'application/json',
      })
    },
    { auto: true },
  ],
  connection: async ({ baseURL }, use) => {
    const origin = new URL(baseURL!)
    let online = true
    const sockets = new Set<Socket>()
    function track(socket: Socket) {
      if (sockets.has(socket)) return
      sockets.add(socket)
      socket.once('close', () => sockets.delete(socket))
    }
    let cutting = false
    let cutResponse: ServerResponse | null = null
    const server = createServer((client, response) => {
      if (!online) {
        client.socket.destroy()
        return
      }
      const upstream = request(
        {
          hostname: origin.hostname,
          port: origin.port,
          path: client.url,
          method: client.method,
          headers: client.headers,
        },
        (incoming) => {
          incoming.on('error', () => response.destroy())
          if (cutting && client.url === '/api/v1/sessions/current') {
            cutting = false
            cutResponse = response
            response.writeHead(200, {
              'Content-Type': 'application/json',
              'Transfer-Encoding': 'chunked',
              'Cache-Control': 'no-store',
            })
            response.write('{')
            incoming.resume()
          } else {
            response.writeHead(incoming.statusCode ?? 502, incoming.headers)
            incoming.pipe(response)
          }
        },
      )
      upstream.on('socket', track)
      upstream.on('error', () => response.destroy())
      client.pipe(upstream)
    })
    server.on('connection', track)
    await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve))
    const address = server.address()
    if (!address || typeof address === 'string') throw new Error('Proxy did not start')
    try {
      await use({
        url: `http://127.0.0.1:${address.port}`,
        disconnect: () => {
          online = false
          for (const socket of sockets) socket.destroy()
        },
        reconnect: () => {
          online = true
        },
        cutNextSessionBody: () => {
          cutting = true
          return () => {
            cutResponse?.destroy()
            cutResponse = null
          }
        },
      })
    } finally {
      for (const socket of sockets) socket.destroy()
      await new Promise<void>((resolve) => server.close(() => resolve()))
    }
  },
  context: async ({ playwright, browserName, connection }, use) => {
    const profile = await mkdtemp(join(tmpdir(), 'phrase-gym-connection-'))
    const context = await playwright[browserName].launchPersistentContext(profile, {
      headless: true,
      baseURL: connection.url,
      serviceWorkers: 'allow',
    })
    try {
      await silencePlayback(context)
      await use(context)
    } finally {
      await context.close()
      await rm(profile, { recursive: true, force: true })
    }
  },
})
test.use({ serviceWorkers: 'allow' })
