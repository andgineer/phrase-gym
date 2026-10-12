import { expect } from '@playwright/test'
import { test } from './fixtures'

test('search fields retain focus through each query response and pending requests', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.goto('/learn/en')
  await expect(page.getByRole('status')).toHaveCount(0)
  for (const [label, query] of [
    ['Search lessons', 'rap'],
    ['Find a phrase', 'greetings'],
  ]) {
    const input = page.getByLabel(label, { exact: true })
    await input.focus()
    for (const character of query) {
      await page.keyboard.type(character)
      await expect(page.getByRole('status')).toHaveCount(0)
      await expect(input).toBeFocused()
    }
    await expect(input).toHaveValue(query)
  }
  await expect(page.locator('.phrase-list li')).toHaveCount(1)
  let release!: () => void
  const gate = new Promise<void>((resolve) => {
    release = resolve
  })
  await page.route('**/api/v1/phrases/search?**', async (route) => {
    if (new URL(route.request().url()).searchParams.get('q') === 'greetingsx') await gate
    await route.continue()
  })
  await page.keyboard.type('x')
  await expect(page.getByRole('status')).toBeVisible()
  await expect(page.getByLabel('Find a phrase', { exact: true })).toBeFocused()
  await expect(page.locator('.lookup')).not.toContainText('No matches')
  release()
  await expect(page.getByRole('status')).toHaveCount(0)
  await expect(page.locator('.lookup')).toContainText('No matches')
})
