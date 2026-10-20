import { defineConfig, devices } from '@playwright/test'

const port = Number(process.env.PHRASE_GYM_BROWSER_TEST_PORT ?? 8941)
const chromiumURL = `http://127.0.0.1:${port}`
const webkitURL = `http://127.0.0.1:${port + 1}`
const preparation = 'preparation-lifecycle.spec.ts'

export default defineConfig({
  testDir: './tests/browser',
  fullyParallel: false,
  workers: 1,
  use: { baseURL: chromiumURL, trace: 'retain-on-failure' },
  webServer: [0, 1, 2, 3].map((offset) => ({
    command:
      offset < 2
        ? `node frontend/tests/browser-server.mjs --port ${port + offset}`
        : `.venv/bin/python -m tests.browser_preparation_server --port ${port + offset}`,
    cwd: '..',
    url: `http://127.0.0.1:${port + offset}/api/v1/settings`,
    reuseExistingServer: false,
    timeout: 120_000,
  })),
  projects: [
    {
      name: 'chromium',
      testIgnore: preparation,
      use: {
        ...devices['Desktop Chrome'],
        baseURL: chromiumURL,
        // Preserve native playback clocks without requiring a runner audio device.
        launchOptions: { args: ['--disable-audio-output'] },
      },
    },
    {
      name: 'webkit',
      testIgnore: preparation,
      use: {
        ...devices['Desktop Safari'],
        baseURL: webkitURL,
      },
    },
    ...(['chromium', 'webkit'] as const).map((browserName, index) => ({
      name: `${browserName}-python-preparation`,
      testMatch: preparation,
      use: {
        browserName,
        baseURL: `http://127.0.0.1:${port + 2 + index}`,
        launchOptions: browserName === 'chromium' ? { args: ['--disable-audio-output'] } : {},
      },
    })),
  ],
})
