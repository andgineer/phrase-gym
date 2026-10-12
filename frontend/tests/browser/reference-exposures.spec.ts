import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

for (const navigate of [false, true]) {
  test(`materials display independently of a held exposure write, navigate=${navigate}`, async ({
    page,
  }) => {
    let release!: () => void
    const gate = new Promise<void>((resolve) => {
      release = resolve
    })
    let eventId = ''
    await page.route('**/api/v1/exposures', async (route) => {
      eventId = route.request().postDataJSON().id
      await gate
      await route.continue()
    })
    await page.goto('/learn/en?q=greetings')
    await page.getByRole('link', { name: /How has your day/ }).click()
    await expect(page.locator('.phrase-card')).toBeVisible()
    await expect(page.getByRole('status')).toHaveText('Saving reference visit…')
    await expect(page.getByRole('alert')).toHaveCount(0)
    await expect.poll(() => eventId).not.toBe('')
    const key = `phrase-gym.reference-exposure.v1.${eventId}`
    const retained = await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), key)
    expect(retained.id).toBe(eventId)
    if (navigate) {
      await page.getByRole('button', { name: 'Back', exact: true }).click()
      await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
      expect(await page.evaluate((key) => localStorage.getItem(key), key)).not.toBeNull()
    }
    const response = page.waitForResponse(
      (response) => response.url().endsWith('/api/v1/exposures') && response.status() === 200,
    )
    release()
    expect((await (await response).json()).id).toBe(eventId)
    await expect.poll(() => page.evaluate((key) => localStorage.getItem(key), key)).toBeNull()
  })
}

test('settings hide bookmarked materials without creating an exposure', async ({ page }) => {
  let exposures = 0
  await page.route('**/api/v1/exposures', async (route) => {
    exposures += 1
    await route.continue()
  })
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await expect(page.locator('.phrase-card')).toBeVisible()
  await expect.poll(() => exposures).toBe(1)
  await page.getByRole('link', { name: 'Phrase Gym' }).click()
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await page.goBack()
  await expect(page.getByLabel('Your native language', { exact: true })).toBeVisible()
  await expect(page.getByRole('status')).toHaveCount(0)
  await expect(page.locator('.phrase-card')).toHaveCount(0)
  expect(exposures).toBe(1)
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await expect(page.locator('.phrase-card')).toBeVisible()
  await expect.poll(() => exposures).toBe(2)
})
