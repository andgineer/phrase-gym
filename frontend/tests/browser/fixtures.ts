import { spawn } from 'node:child_process'
import { resolve } from 'node:path'
import { expect, test as base } from '@playwright/test'
import { createSilentContext } from './silent-playback'

type WorkerServer = { url: string; logs: () => string }

export const test = base.extend<
  { server: void; coachEnabled: boolean; coachRoute: 'claude' | 'gemini' },
  { workerServer: WorkerServer }
>({
  coachEnabled: [true, { option: true }],
  coachRoute: ['claude', { option: true }],
  workerServer: [
    async ({}, use, workerInfo) => {
      const url = workerInfo.project.use.baseURL
      if (!url) throw new Error('Browser tests require a baseURL')
      const server = spawn(
        'uv',
        ['run', 'python', '-m', 'tests.browser_server', new URL(url).port],
        {
          cwd: resolve(import.meta.dirname, '../../..'),
          stdio: ['ignore', 'pipe', 'pipe'],
        },
      )
      let logs = ''
      let startupError: Error | undefined
      server.stdout.on('data', (data) => (logs += data.toString()))
      server.stderr.on('data', (data) => (logs += data.toString()))
      server.on('error', (error) => (startupError = error))
      const stopped = new Promise<void>((resolve) => server.once('close', () => resolve()))
      try {
        await expect
          .poll(
            async () => {
              if (startupError) throw startupError
              if (server.exitCode !== null) throw new Error(`Test server exited: ${logs}`)
              if (!logs.includes('Uvicorn running on')) return false
              return fetch(`${url}/api/health`)
                .then((response) => response.ok)
                .catch(() => false)
            },
            { timeout: 10_000, intervals: [10, 25, 50, 100] },
          )
          .toBe(true)
        await use({ url, logs: () => logs })
      } finally {
        server.kill('SIGTERM')
        const deadline = setTimeout(() => server.kill('SIGKILL'), 5_000)
        try {
          await stopped
        } finally {
          clearTimeout(deadline)
        }
      }
    },
    { scope: 'worker' },
  ],
  server: [
    async ({ workerServer, coachEnabled, coachRoute }, use, testInfo) => {
      const offset = workerServer.logs().length
      try {
        const response = await fetch(`${workerServer.url}/__test/reset`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ coach_enabled: coachEnabled, coach_route: coachRoute }),
        })
        expect(response.ok).toBe(true)
        await use()
      } finally {
        if (testInfo.status !== testInfo.expectedStatus)
          await testInfo.attach('server.log', {
            body: workerServer.logs().slice(offset),
            contentType: 'text/plain',
          })
      }
    },
    { auto: true },
  ],
  context: async ({ browser, viewport, hasTouch, isMobile }, use) => {
    const context = await createSilentContext(browser, { viewport, hasTouch, isMobile })
    await use(context)
    await context.close()
  },
})
export { expect }
