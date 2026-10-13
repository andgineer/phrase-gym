import { createServer, connect } from 'node:net'
import type { Socket } from 'node:net'
import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { test as base } from './fixtures'

type Connection = { url: string; disconnect: () => void; reconnect: () => void }
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
    const server = createServer((client) => {
      if (!online) {
        client.destroy()
        return
      }
      const upstream = connect(Number(origin.port), origin.hostname)
      for (const socket of [client, upstream]) {
        sockets.add(socket)
        socket.on('error', () => {
          client.destroy()
          upstream.destroy()
        })
        socket.on('close', () => sockets.delete(socket))
      }
      client.pipe(upstream).pipe(client)
    })
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
      await use(context)
    } finally {
      await context.close()
      await rm(profile, { recursive: true, force: true })
    }
  },
})
test.use({ serviceWorkers: 'allow' })
