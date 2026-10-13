import { expect } from '@playwright/test'
import { test } from './fixtures'

for (const status of [200, 503]) {
  test(`initial settings read stays single during navigation and recovers after ${status}`, async ({
    page,
    request,
  }) => {
    const phrases = await (
      await request.get('/api/v1/phrases/search?language=en&q=greetings')
    ).json()
    let release!: () => void
    const gate = new Promise<void>((resolve) => {
      release = resolve
    })
    let reads = 0
    let held = false
    await page.route('**/api/v1/settings', async (route) => {
      if (route.request().method() !== 'GET') {
        await route.continue()
        return
      }
      reads += 1
      if (reads !== 1) {
        await route.continue()
        return
      }
      const response = await route.fetch()
      held = true
      await gate
      if (status === 200) await route.fulfill({ response })
      else
        await route.fulfill({
          status,
          body: '{"detail":"Settings unavailable"}',
          contentType: 'application/json',
        })
    })
    await page.goto(`/learn/en/phrases/${phrases.items[0].id}`)
    await expect.poll(() => held).toBe(true)
    await page.getByRole('link', { name: 'Phrase Gym' }).click()
    await expect(page).toHaveURL(/\/learn\/en$/)
    await page.evaluate(() => new Promise(requestAnimationFrame))
    expect(reads).toBe(1)
    await expect(page.getByRole('status')).toHaveText('Loading…')
    await expect(page.getByRole('heading', { name: 'Make it yours' })).toHaveCount(0)
    release()
    if (status === 503) {
      await expect(page.getByTestId('connection-required')).toBeVisible()
      await page.getByTestId('connection-required').getByRole('button', { name: 'Retry' }).click()
    }
    await expect(page.getByRole('heading', { name: 'Make it yours' })).toBeVisible()
    await page.getByLabel('Your native language', { exact: true }).selectOption('en')
    await page.getByRole('button', { name: 'Save', exact: true }).click()
    await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
    expect((await (await request.get('/api/v1/settings')).json()).native_language).toBe('en')
    expect(reads).toBe(status === 200 ? 1 : 2)
    await page.getByLabel('Find a phrase', { exact: true }).fill('greetings')
    await expect(page.locator('.phrase-list li')).toHaveCount(1)
    await expect(page.getByRole('heading', { name: 'Make it yours' })).toHaveCount(0)
    expect(reads).toBe(status === 200 ? 1 : 2)
  })
}
