import { test as base } from '@playwright/test'

export const test = base.extend<{ isolatedLibrary: void }>({
  isolatedLibrary: [
    async ({ request }, use) => {
      const response = await request.post('/_test/reset')
      if (!response.ok()) throw new Error(`Fixture reset failed (${response.status()})`)
      await use()
    },
    { auto: true },
  ],
})
