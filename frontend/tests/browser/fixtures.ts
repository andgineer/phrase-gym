import { spawn } from 'node:child_process'
import { resolve } from 'node:path'
import { expect, test as base } from '@playwright/test'
import { createSilentContext } from './silent-playback'

export const test = base.extend<{ server: void; coachEnabled: boolean }>({
  coachEnabled: [true, { option: true }],
  server: [
    async ({ baseURL, coachEnabled }, use, testInfo) => {
      if (!baseURL) throw new Error('Browser tests require a baseURL')
      const server = spawn(
        'uv',
        ['run', 'python', 'tests/browser_server.py', new URL(baseURL).port],
        {
          cwd: resolve(import.meta.dirname, '../../..'),
          env: { ...process.env, PHRASE_GYM_BROWSER_NO_COACH: coachEnabled ? '0' : '1' },
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
              return fetch(`${baseURL}/api/health`)
                .then((response) => response.ok)
                .catch(() => false)
            },
            { timeout: 10_000 },
          )
          .toBe(true)
        await use()
      } finally {
        server.kill('SIGTERM')
        const deadline = setTimeout(() => server.kill('SIGKILL'), 5_000)
        try {
          await stopped
        } finally {
          clearTimeout(deadline)
        }
        if (testInfo.status !== testInfo.expectedStatus)
          await testInfo.attach('server.log', { body: logs, contentType: 'text/plain' })
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
