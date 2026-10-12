import { expect } from '@playwright/test'
import { test } from './fixtures'
import { publishPagedLibrary } from './paged-library'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning', last_learning_language: 'en' },
  })
})

for (const view of ['lookup', 'lesson', 'lessons']) {
  for (const outcome of ['retry', 'navigate']) {
    test(`${view} pagination handles ${outcome} without replacing or duplicating items`, async ({
      page,
      request,
    }) => {
      const { token, lessonId } = await publishPagedLibrary(request)
      await page.goto(
        view === 'lesson'
          ? `/learn/en/lessons/${lessonId}`
          : `/learn/en?${view === 'lessons' ? 'lessonQ' : 'q'}=${token}`,
      )
      const section =
        view === 'lessons'
          ? page.locator('.session-entry')
          : view === 'lookup'
            ? page.locator('.lookup')
            : page.locator('.card')
      const rows =
        view === 'lessons'
          ? section.locator('option[value]:not([value=""])')
          : section.locator('.phrase-list a')
      const more = section.getByRole('button', { name: 'Show more', exact: true })
      await expect(rows).toHaveCount(50)
      let release!: () => void
      const gate = new Promise<void>((resolve) => {
        release = resolve
      })
      let requests = 0
      await page.route('**/api/v1/**', async (route) => {
        if (new URL(route.request().url()).searchParams.get('offset') !== '50') {
          await route.continue()
          return
        }
        requests += 1
        if (outcome === 'retry' && requests === 1) {
          await route.fulfill({
            status: 503,
            contentType: 'application/json',
            body: '{"detail":"Page unavailable"}',
          })
        } else if (outcome === 'navigate') {
          const response = await route.fetch()
          await gate
          await route.fulfill({ response })
        } else await route.continue()
      })
      await more.click()
      await expect.poll(() => requests).toBe(1)
      if (outcome === 'retry') {
        await expect(page.getByRole('alert')).toContainText('Page unavailable')
        await expect(rows).toHaveCount(50)
        await expect(more).toBeEnabled()
        await more.click()
        await expect(rows).toHaveCount(100)
        if (view !== 'lessons') {
          await more.click()
          await expect(rows).toHaveCount(120)
        }
        await expect(more).toHaveCount(0)
        await expect(page.getByRole('alert')).toHaveCount(0)
        const identities = await rows.evaluateAll((elements) =>
          elements.map((element) => element.getAttribute('href') ?? element.getAttribute('value')),
        )
        expect(new Set(identities).size).toBe(view === 'lessons' ? 100 : 120)
        expect(requests).toBe(2)
      } else {
        await expect(more).toBeDisabled()
        await page.getByLabel('Learning language', { exact: true }).selectOption('sr')
        await expect(page.getByRole('heading', { name: 'Biblioteka', exact: true })).toBeVisible()
        await expect(page.locator('.phrase-list')).toContainText('Drago mi je.')
        const response = page.waitForResponse(
          (response) => new URL(response.url()).searchParams.get('offset') === '50',
        )
        release()
        await (await response).finished()
        await page.evaluate(() => new Promise(requestAnimationFrame))
        await expect(page.locator('.phrase-list li')).toHaveCount(1)
        await expect(page.locator('.session-entry option')).toHaveCount(2)
        await expect(page.getByRole('alert')).toHaveCount(0)
      }
    })
  }
}
