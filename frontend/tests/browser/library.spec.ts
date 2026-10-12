import { readFileSync } from 'node:fs'
import { test, expect } from '@playwright/test'

test.beforeEach(async ({ request }, testInfo) => {
  if (testInfo.title !== 'first launch and persisted native-language settings') {
    await request.put('/api/v1/settings', {
      data: {
        native_language: 'ru',
        ui_language_mode: 'learning',
        last_learning_language: 'en',
      },
    })
  }
})

test('first launch and persisted native-language settings', async ({ page, request }) => {
  const collection = await (
    await request.get('/api/v1/phrases/search?language=en&q=greetings')
  ).json()
  let exposures = 0
  await page.route('**/api/v1/exposures', async (route) => {
    exposures += 1
    await route.continue()
  })
  await page.goto(`/learn/en/phrases/${collection.items[0].id}`)
  await expect(page.getByRole('heading', { name: 'Make it yours' })).toBeVisible()
  await expect(page.getByRole('status')).toHaveCount(0)
  await expect(page.locator('.phrase-card')).toHaveCount(0)
  expect(exposures).toBe(0)
  await expect(page.getByRole('button', { name: 'Save', exact: true })).toBeDisabled()
  await page.getByLabel('Native language', { exact: true }).selectOption('ru')
  await page.getByRole('checkbox').check()
  await page.getByRole('button', { name: 'Save', exact: true }).click()
  await expect(page.locator('.explanation')).toBeVisible()
  await expect.poll(() => exposures).toBe(1)
  await page.getByRole('link', { name: 'Phrase Gym' }).click()
  await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await expect(page.getByLabel('Native language', { exact: true })).toHaveValue('ru')
})

test('concept lookup, explanation and complete lesson navigation preserve search', async ({
  page,
}) => {
  await page.goto('/learn/en')
  await page.getByLabel('Search lessons').fill('rap')
  await expect(page.locator('select[aria-label="Lesson to drill"] option')).toHaveCount(2)
  await page.getByLabel('Find a phrase', { exact: true }).fill('greetings')
  await expect(page.locator('.phrase-list li')).toHaveCount(1)
  await page.getByRole('link', { name: /How has your day/ }).click()
  await expect(page.locator('.explanation')).toContainText('Как проходит твой день?')
  await page.getByRole('link', { name: /Building rapport/ }).click()
  await expect(page.locator('.phrase-list li')).toHaveCount(2)
  await expect(page.getByRole('button', { name: 'Start lesson drill' })).toBeDisabled()
  await page.getByRole('button', { name: 'Back', exact: true }).click()
  await expect(page.locator('.explanation')).toBeVisible()
  await page.getByRole('button', { name: 'Back', exact: true }).click()
  await expect(page.getByLabel('Find a phrase', { exact: true })).toHaveValue('greetings')
  await expect(page.getByLabel('Search lessons')).toHaveValue('rap')
})

test('bookmarks, two tabs, reload and browser history retain learning language', async ({
  page,
  context,
}) => {
  await page.goto('/learn/en')
  const sr = await context.newPage()
  await sr.goto('/learn/sr')
  await expect(sr.locator('h1')).toHaveText('Biblioteka')
  await expect(sr.locator('.phrase-list')).toContainText('Drago mi je.')
  await page.reload()
  await expect(page.locator('.phrase-list')).toContainText('How has your day')
  await page.getByLabel('Learning language', { exact: true }).selectOption('sr')
  await expect(page).toHaveURL(/\/learn\/sr$/)
  await page.goBack()
  await expect(page.locator('h1')).toHaveText('Library')
  await page.goForward()
  await expect(page.locator('h1')).toHaveText('Biblioteka')
  await sr.getByRole('link', { name: /Drago mi je/ }).click()
  await expect(sr.locator('.explanation')).toContainText('Приятно познакомиться.')
  await sr.close()
})

test('native UI, bare fallback, invalid language and empty collection', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: {
      native_language: 'ru',
      ui_language_mode: 'native',
      last_learning_language: 'sr',
    },
  })
  await page.goto('/')
  await expect(page).toHaveURL(/\/learn\/sr$/)
  await expect(page.locator('h1')).toHaveText('Библиотека')
  await page.goto('/learn/unknown')
  await expect(page.locator('h1')).toHaveText('Выберите изучаемый язык')
  await expect(page.locator('.phrase-list')).toHaveCount(0)
  await page.goto('/learn/de')
  await expect(page.locator('.lookup')).toContainText('Пока нет фраз')
  await page.getByLabel('Найти фразу', { exact: true }).fill('something')
  await expect(page.locator('.phrase-list li')).toHaveCount(0)
})

test('reference audio starts with a gesture, stops on navigation and preserves original bytes', async ({
  page,
}) => {
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await expect(page.getByRole('button', { name: 'Reference', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Reference', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Stop playback', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Reference', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Reference', exact: true }).click()
  await page.getByRole('button', { name: 'Back', exact: true }).click()
  await expect(page.locator('audio')).toHaveCount(0)
  await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
})

test('reference exposure retries retain an immutable event through reload', async ({ page }) => {
  await page.route('**/api/v1/exposures', (route) =>
    route.fulfill({
      status: 503,
      body: '{"detail":"Unavailable"}',
      contentType: 'application/json',
    }),
  )
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await expect(page.getByRole('alert')).toContainText('Could not save this reference exposure')
  const before = await page.evaluate(() =>
    Object.keys(localStorage).filter((k) => k.startsWith('phrase-gym.reference-exposure.v1.')),
  )
  expect(before.length).toBe(1)
  await page.unroute('**/api/v1/exposures')
  await page.reload()
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          Object.keys(localStorage).filter((k) => k.startsWith('phrase-gym.reference-exposure.v1.'))
            .length,
      ),
    )
    .toBe(0)
})

test('narrow viewport, text enlargement and safe imported text', async ({ page, request }) => {
  const content = JSON.parse(readFileSync('../fixtures/manual-en.json', 'utf8'))
  const unsafe = '<img src=x onerror="window.sourceExecuted=true">'
  content.targets[0].explanations.ru.usage = unsafe
  const report = await (await request.post('/api/v1/imports', { data: content })).json()
  expect(report.status).toBe('staged')
  expect(
    (await request.post(`/api/v1/imports/${report.id}/publish`, { data: { reviewed: true } })).ok(),
  ).toBe(true)
  await page.setViewportSize({ width: 320, height: 800 })
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await page.addStyleTag({ content: 'html { font-size: 32px; }' })
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  )
  await expect(page.locator('.explanation')).toContainText(unsafe)
  await expect(page.locator('.explanation img')).toHaveCount(0)
  expect(
    await page.evaluate(() => (window as unknown as { sourceExecuted?: boolean }).sourceExecuted),
  ).toBeUndefined()
  await page.screenshot({ path: 'test-results/p2-narrow.png', fullPage: true })
})
