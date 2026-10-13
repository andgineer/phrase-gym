import { defineConfig, devices } from '@playwright/test'

export default defineConfig({
  testDir: './tests/browser',
  fullyParallel: false,
  workers: 1,
  use: { baseURL: 'http://127.0.0.1:8941', trace: 'retain-on-failure' },
  webServer: [8941, 8942].map((port) => ({
    command: `.venv/bin/python -m tests.browser_server --port ${port}`,
    cwd: '..',
    url: `http://127.0.0.1:${port}/api/v1/settings`,
    reuseExistingServer: false,
  })),
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'], baseURL: 'http://127.0.0.1:8941' } },
    { name: 'webkit', use: { ...devices['Desktop Safari'], baseURL: 'http://127.0.0.1:8942' } },
  ],
})
