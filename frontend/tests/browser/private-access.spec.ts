import { createSilentContext } from './silent-playback'
import { expect } from '@playwright/test'
import { test } from './fixtures'

test('local practice starts without application sign-in or cookies', async ({
  browser,
  baseURL,
}) => {
  const context = await createSilentContext(browser, { baseURL })
  try {
    const page = await context.newPage()
    expect((await context.request.get('/api/v1/settings')).status()).toBe(200)
    await page.goto('/learn/en')
    await expect(page.getByLabel('Learner access token')).toHaveCount(0)
    expect(await context.cookies()).toEqual([])
    await page.reload()
    expect((await context.request.get('/api/v1/profile')).status()).toBe(200)
    expect((await context.request.get('/login')).status()).toBe(404)
  } finally {
    await context.close()
  }
})
