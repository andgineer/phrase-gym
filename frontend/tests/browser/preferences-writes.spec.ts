import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning', last_learning_language: 'en' },
  })
})

for (const lastLanguage of ['sr', 'de']) {
  for (const status of [200, 503]) {
    test(`ordered language writes restore ${lastLanguage} after an old ${status} response`, async ({
      page,
      request,
    }) => {
      let release!: () => void
      const gate = new Promise<void>((resolve) => {
        release = resolve
      })
      const writes: string[] = []
      await page.route('**/api/v1/settings/language', async (route) => {
        writes.push(route.request().postDataJSON().last_learning_language)
        if (writes.length === 1) {
          await gate
          if (status === 503) {
            await route.fulfill({
              status,
              body: '{"detail":"Old preference unavailable"}',
              contentType: 'application/json',
            })
            return
          }
        }
        await route.continue()
      })
      await page.goto('/learn/en')
      await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
      await expect.poll(() => writes.length).toBe(1)
      await page.locator('header select').selectOption('sr')
      await expect(page.getByRole('heading', { name: 'Biblioteka', exact: true })).toBeVisible()
      if (lastLanguage === 'de') {
        await page.locator('header select').selectOption('de')
        await expect(page.getByRole('heading', { name: 'Bibliothek', exact: true })).toBeVisible()
      }
      expect(writes).toEqual(['en'])
      const oldResponse = page.waitForResponse(
        (response) =>
          response.url().endsWith('/api/v1/settings/language') &&
          response.request().postDataJSON().last_learning_language === 'en',
      )
      release()
      await (await oldResponse).finished()
      await expect
        .poll(
          async () => (await (await request.get('/api/v1/settings')).json()).last_learning_language,
        )
        .toBe(lastLanguage)
      expect(writes).toEqual(['en', lastLanguage])
      await page.goto('/')
      await expect(page).toHaveURL(new RegExp(`/learn/${lastLanguage}$`))
    })
  }
}

test('overlapping settings saves preserve submission order and the latest native preference', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  let release!: () => void
  const gate = new Promise<void>((resolve) => {
    release = resolve
  })
  const writes: string[] = []
  await page.route('**/api/v1/settings', async (route) => {
    if (route.request().method() === 'PUT') {
      writes.push(route.request().postDataJSON().native_language)
      if (writes.length === 1) await gate
    }
    await route.continue()
  })
  await page.getByLabel('Native language', { exact: true }).selectOption('en')
  await page.getByRole('button', { name: 'Save', exact: true }).click()
  await expect.poll(() => writes.length).toBe(1)
  await page.getByLabel('Native language', { exact: true }).selectOption('de')
  await page.getByRole('button', { name: 'Save', exact: true }).click()
  expect(writes).toEqual(['en'])
  release()
  await expect
    .poll(async () => (await (await request.get('/api/v1/settings')).json()).native_language)
    .toBe('de')
  await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
  expect(writes).toEqual(['en', 'de'])
  await page.reload()
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await expect(page.getByLabel('Native language', { exact: true })).toHaveValue('de')
})
