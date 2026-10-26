import { defineConfig, devices } from '@playwright/test'

const port = Number(process.env.PHRASE_GYM_BROWSER_TEST_PORT ?? 8941)
export default defineConfig({
  testDir: './tests/browser',
  fullyParallel: false,
  workers: 2,
  use: { trace: 'retain-on-failure' },
  webServer: [0, 1].map((offset) => ({
    command: `uv run python tests/browser_server.py ${port + offset}`,
    cwd: '..',
    url: `http://127.0.0.1:${port + offset}/api/health`,
    reuseExistingServer: false,
    timeout: 120_000,
  })),
  projects: [
    {
      name: 'chromium',
      workers: 1,
      use: {
        ...devices['Desktop Chrome'],
        baseURL: `http://127.0.0.1:${port}`,
        launchOptions: { args: ['--disable-audio-output'] },
      },
    },
    {
      name: 'webkit',
      workers: 1,
      use: { ...devices['Desktop Safari'], baseURL: `http://127.0.0.1:${port + 1}` },
    },
  ],
})
