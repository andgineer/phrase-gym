import { defineConfig, devices } from '@playwright/test'

const port = Number(process.env.PHRASE_GYM_BROWSER_TEST_PORT ?? 8941)
export default defineConfig({
  testDir: './tests/browser',
  fullyParallel: false,
  workers: 2,
  use: { trace: 'retain-on-failure' },
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
