import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning', last_learning_language: 'en' },
  })
})

for (const language of ['sr', 'de']) {
  test(`a delayed full save preserves newer ${language} fallback through browser history`, async ({
    page,
    request,
  }) => {
    await page.goto('/learn/invalid')
    await page.getByRole('link', { name: 'English', exact: true }).click()
    await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
    let releaseSave!: () => void
    const saveGate = new Promise<void>((resolve) => {
      releaseSave = resolve
    })
    let heldSave = false
    await page.route('**/api/v1/settings', async (route) => {
      if (route.request().method() !== 'PUT') {
        await route.continue()
        return
      }
      const response = await route.fetch()
      heldSave = true
      await saveGate
      await route.fulfill({ response })
    })
    await page.getByRole('button', { name: 'Settings', exact: true }).click()
    await page.getByLabel('Your native language', { exact: true }).selectOption('de')
    await page.getByRole('button', { name: 'Save', exact: true }).click()
    await expect.poll(() => heldSave).toBe(true)
    await page.getByRole('button', { name: 'Settings', exact: true }).click()
    await page.locator('header select').selectOption(language)
    await expect(
      page.getByRole('heading', {
        name: language === 'sr' ? 'Biblioteka' : 'Bibliothek',
        exact: true,
      }),
    ).toBeVisible()
    await expect(page.getByRole('status')).toHaveCount(0)
    let releaseCatalog!: () => void
    const catalogGate = new Promise<void>((resolve) => {
      releaseCatalog = resolve
    })
    let heldCatalog = false
    await page.route('**/api/v1/phrases/search?**', async (route) => {
      if (new URL(route.request().url()).searchParams.get('language') === language) {
        heldCatalog = true
        await catalogGate
      }
      await route.continue()
    })
    releaseSave()
    await expect.poll(() => heldCatalog).toBe(true)
    await expect
      .poll(
        async () => (await (await request.get('/api/v1/settings')).json()).last_learning_language,
      )
      .toBe(language)
    await page.evaluate(() => history.go(-2))
    await expect(page).toHaveURL(/\/learn\/invalid$/)
    await expect(page.getByRole('status')).toHaveCount(0)
    await page.getByRole('link', { name: 'Phrase Gym' }).click()
    await expect(page).toHaveURL(new RegExp(`/learn/${language}$`))
    releaseCatalog()
    await expect(page.getByRole('status')).toHaveCount(0)
    await expect
      .poll(
        async () => (await (await request.get('/api/v1/settings')).json()).last_learning_language,
      )
      .toBe(language)
  })
}
