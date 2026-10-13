import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

for (const change of ['query', 'language']) {
  for (const status of [200, 503]) {
    test(`a stale ${status} search response cannot overwrite a newer ${change}`, async ({
      page,
    }) => {
      await page.goto('/learn/en')
      await expect(page.locator('.phrase-list li')).toHaveCount(3)
      let release!: () => void
      const gate = new Promise<void>((resolve) => {
        release = resolve
      })
      let held = false
      await page.route('**/api/v1/phrases/search?**', async (route) => {
        if (new URL(route.request().url()).searchParams.get('q') !== 'coffee') {
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
            contentType: 'application/json',
            body: '{"detail":"Old search failed"}',
          })
      })
      await page.getByLabel('Find a phrase', { exact: true }).fill('coffee')
      await expect.poll(() => held).toBe(true)
      await expect(page.getByRole('status')).toBeVisible()
      if (change === 'query')
        await page.getByLabel('Find a phrase', { exact: true }).fill('greetings')
      else await page.getByLabel('Learning language', { exact: true }).selectOption('sr')
      const expected = change === 'query' ? 'How has your day' : 'Drago mi je.'
      await expect(page.locator('.phrase-list')).toContainText(expected)
      const response = page.waitForResponse(
        (response) => new URL(response.url()).searchParams.get('q') === 'coffee',
      )
      release()
      await (await response).finished()
      await page.evaluate(() => new Promise(requestAnimationFrame))
      if (status === 503) {
        await expect(page.getByTestId('connection-required')).toBeVisible()
        await page
          .getByTestId('connection-required')
          .getByRole('button', { name: /Retry|Pokušaj ponovo/ })
          .click()
      }
      await expect(page.locator('.phrase-list li')).toHaveCount(1)
      await expect(page.locator('.phrase-list')).toContainText(expected)
      await expect(page.getByRole('alert')).toHaveCount(0)
      await expect(page.getByRole('status')).toHaveCount(0)
    })
  }
}

test('a failed search offers retry without claiming no matches', async ({ page }) => {
  await page.goto('/learn/en')
  await expect(page.locator('.phrase-list li')).toHaveCount(3)
  let failed = false
  await page.route('**/api/v1/phrases/search?**', async (route) => {
    if (!failed && new URL(route.request().url()).searchParams.get('q') === 'greetings') {
      failed = true
      await route.fulfill({
        status: 503,
        contentType: 'application/json',
        body: '{"detail":"Temporary search failure"}',
      })
    } else await route.continue()
  })
  await page.getByLabel('Find a phrase', { exact: true }).fill('greetings')
  await expect(page.getByTestId('connection-required')).toBeVisible()
  await expect(page.locator('.lookup')).not.toContainText('No matches')
  await expect(page.locator('.phrase-list li')).toHaveCount(0)
  await expect(page.getByLabel('Find a phrase', { exact: true })).toHaveValue('greetings')
  await page
    .getByTestId('connection-required')
    .getByRole('button', { name: /Retry|Pokušaj ponovo/ })
    .click()
  await expect(page.locator('.phrase-list li')).toHaveCount(1)
  await expect(page.getByRole('alert')).toHaveCount(0)
})
