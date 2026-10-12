import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

for (const language of ['sr', 'de']) {
  test(`a delayed Settings save cannot change the ${language} URL or device fallback`, async ({
    page,
    request,
  }) => {
    await page.goto('/learn/en')
    let release!: () => void
    const gate = new Promise<void>((resolve) => {
      release = resolve
    })
    let held = false
    const mutations: string[] = []
    page.on('request', (request) => {
      if (request.url().includes('/api/v1/settings') && request.method() !== 'GET')
        mutations.push(request.method())
    })
    await page.route('**/api/v1/settings', async (route) => {
      if (route.request().method() !== 'PUT') return route.continue()
      const response = await route.fetch()
      held = true
      await gate
      await route.fulfill({ response })
    })
    await page.getByRole('button', { name: 'Settings', exact: true }).click()
    await page.getByLabel('Your native language', { exact: true }).selectOption('de')
    await page.getByRole('button', { name: 'Save', exact: true }).click()
    await expect.poll(() => held).toBe(true)
    await page.locator('header select').selectOption(language)
    await expect(page).toHaveURL(new RegExp(`/learn/${language}$`))
    release()
    await expect(
      page.getByRole('heading', {
        name: language === 'sr' ? 'Biblioteka' : 'Bibliothek',
        exact: true,
      }),
    ).toBeVisible()
    expect(await (await request.get('/api/v1/settings')).json()).toEqual({
      native_language: 'de',
      ui_language_mode: 'learning',
    })
    expect(mutations).toEqual(['PUT'])
    await page.goto('/')
    await expect(page).toHaveURL(new RegExp(`/learn/${language}$`))
  })
}

test('bare routes restore only on this device; explicit tabs remain independent', async ({
  page,
  context,
  browser,
  request,
}) => {
  await page.goto('/')
  await expect(
    page.getByRole('heading', { name: 'Выберите изучаемый язык', exact: true }),
  ).toBeVisible()
  await page.getByRole('link', { name: 'English', exact: true }).click()
  const second = await context.newPage()
  await second.goto('/learn/sr')
  await expect(second.getByRole('heading', { name: 'Biblioteka', exact: true })).toBeVisible()
  await expect(page).toHaveURL(/\/learn\/en$/)
  await page.reload()
  await expect(page).toHaveURL(/\/learn\/en$/)
  await page.goto('/')
  await expect(page).toHaveURL(/\/learn\/en$/)
  const otherDevice = await browser.newContext({ baseURL: new URL(page.url()).origin })
  try {
    const other = await otherDevice.newPage()
    await other.goto('/')
    await expect(
      other.getByRole('heading', { name: 'Выберите изучаемый язык', exact: true }),
    ).toBeVisible()
    expect(await (await request.get('/api/v1/settings')).json()).toEqual({
      native_language: 'ru',
      ui_language_mode: 'learning',
    })
  } finally {
    await otherDevice.close()
  }
})

for (const blocked of [false, true]) {
  test(`an ${blocked ? 'unavailable' : 'invalid'} device preference allows language selection`, async ({
    page,
  }) => {
    await page.addInitScript((blocked) => {
      if (blocked) {
        Object.defineProperty(window, 'localStorage', {
          get() {
            throw new Error('Storage unavailable')
          },
        })
      } else {
        localStorage.setItem('phrase-gym.last-learning-language.v1', 'unsupported')
      }
    }, blocked)
    await page.goto('/')
    await expect(
      page.getByRole('heading', { name: 'Выберите изучаемый язык', exact: true }),
    ).toBeVisible()
    await page.getByRole('link', { name: 'English', exact: true }).click()
    await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
    await page.reload()
    await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
  })
}
