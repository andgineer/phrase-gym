import { defineConfig, devices } from '@playwright/test'

const port = Number(process.env.PHRASE_GYM_BROWSER_TEST_PORT ?? 8941)
const chromiumURL = `http://127.0.0.1:${port}`
const webkitURL = `http://127.0.0.1:${port + 1}`

export default defineConfig({
  testDir: './tests/browser',
  fullyParallel: false,
  workers: 1,
  use: { baseURL: chromiumURL, trace: 'retain-on-failure' },
  webServer: [port, port + 1].map((port) => ({
    command: `.venv/bin/python -m tests.browser_server --port ${port}`,
    cwd: '..',
    url: `http://127.0.0.1:${port}/api/v1/settings`,
    reuseExistingServer: false,
  })),
  projects: [
    {
      name: 'chromium',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: chromiumURL,
        // Preserve native playback clocks without requiring a runner audio device.
        launchOptions: { args: ['--disable-audio-output'] },
      },
    },
    { name: 'webkit', use: { ...devices['Desktop Safari'], baseURL: webkitURL } },
  ],
})
