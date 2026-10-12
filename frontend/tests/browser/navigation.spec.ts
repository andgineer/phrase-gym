import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning', last_learning_language: 'en' },
  })
})

test('changing the lesson filter clears the previous visible selection and link', async ({
  page,
}) => {
  await page.goto('/learn/en')
  const selector = page.getByRole('combobox', { name: 'Lesson to drill', exact: true })
  await selector.selectOption({ label: 'Building rapport' })
  await expect(page.getByRole('link', { name: 'Open selected lesson' })).toBeVisible()
  await page.getByLabel('Search lessons', { exact: true }).fill('plans')
  await expect(selector.locator('option')).toHaveCount(2)
  await expect(selector).toHaveValue('')
  await expect(page.getByRole('link', { name: 'Open selected lesson' })).toHaveCount(0)
  await selector.selectOption({ label: 'Making plans' })
  await page.getByRole('link', { name: 'Open selected lesson' }).click()
  await expect(page.getByRole('heading', { name: 'Making plans' })).toBeVisible()
})

for (const kind of ['phrases', 'lessons']) {
  test(`a direct Serbian ${kind} bookmark updates bare-route restoration`, async ({
    page,
    request,
  }) => {
    const collection = await (
      await request.get(
        kind === 'phrases' ? '/api/v1/phrases/search?language=sr' : '/api/v1/lessons?language=sr',
      )
    ).json()
    await page.goto(`/learn/sr/${kind}/${collection.items[0].id}`)
    await expect(page.getByRole('status')).toHaveCount(0)
    await expect
      .poll(
        async () => (await (await request.get('/api/v1/settings')).json()).last_learning_language,
      )
      .toBe('sr')
    await page.goto('/')
    await expect(page).toHaveURL(/\/learn\/sr$/)
    await expect(page.getByRole('heading', { name: 'Biblioteka', exact: true })).toBeVisible()
  })
}
