import { spawn, type ChildProcess } from 'node:child_process'
import { mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { createServer } from 'node:net'
import { test as base } from './capture-fixtures'

const token = 'synthetic-browser-00000000000000000000'
export const test = base.extend<{ preview: { url: string; restart: () => Promise<void> } }>({
  preview: async ({}, use) => {
    const directory = await mkdtemp(join(tmpdir(), 'phrase-gym-preview-'))
    const credentials = join(directory, 'credentials.json')
    await writeFile(
      credentials,
      JSON.stringify([
        {
          actor: 'synthetic-preview',
          token,
          capabilities: ['learner', 'worker', 'material_submit', 'profile_read', 'profile_update'],
        },
      ]),
      { mode: 0o600 },
    )
    const allocator = createServer()
    await new Promise<void>((resolve) => allocator.listen(0, '127.0.0.1', resolve))
    const address = allocator.address()
    if (!address || typeof address === 'string') throw new Error('No preview test port')
    const port = address.port
    await new Promise<void>((resolve, reject) =>
      allocator.close((error) => (error ? reject(error) : resolve())),
    )
    const url = `http://127.0.0.1:${port}`
    let process: ChildProcess | null = null
    let output = ''
    async function stop() {
      const child = process
      if (!child || child.exitCode !== null) return
      await new Promise<void>((resolve) => {
        child.once('exit', () => resolve())
        child.kill('SIGTERM')
      })
      process = null
    }
    async function start() {
      process = spawn(
        resolve('../target/debug/phrase-gym-server'),
        [
          'serve',
          '--credentials-file',
          credentials,
          '--data-dir',
          directory,
          '--client-dir',
          resolve('dist'),
          '--port',
          String(port),
        ],
        { stdio: ['ignore', 'pipe', 'pipe'] },
      )
      process.stderr?.on('data', (chunk) => {
        output += chunk.toString()
      })
      process.stdout?.on('data', (chunk) => {
        output += chunk.toString()
      })
      const deadline = Date.now() + 15000
      while (Date.now() < deadline) {
        if (process.exitCode !== null) throw new Error(`Preview Server exited: ${output}`)
        if (
          await fetch(`${url}/api/v1/settings`, { headers: { Authorization: `Bearer ${token}` } })
            .then((r) => r.ok)
            .catch(() => false)
        )
          return
        await new Promise((resolve) => setTimeout(resolve, 50))
      }
      throw new Error(`Preview Server did not start: ${output}`)
    }
    try {
      await start()
      await use({
        url,
        restart: async () => {
          await stop()
          await start()
        },
      })
    } finally {
      await stop()
      await rm(directory, { recursive: true, force: true })
    }
  },
  baseURL: async ({ preview }, use) => use(preview.url),
  isolatedLibrary: async ({}, use) => use(),
})
