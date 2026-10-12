import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

for (const status of [200, 503]) {
  test(`a pending Settings save is exclusive and a ${status} response releases controls`, async ({
    page,
    request,
  }) => {
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Settings', exact: true }).click()
    let release!: () => void
    const gate = new Promise<void>((resolve) => {
      release = resolve
    })
    let writes = 0
    await page.route('**/api/v1/settings', async (route) => {
      if (route.request().method() !== 'PUT') return route.continue()
      writes += 1
      if (writes === 1) {
        await gate
        if (status === 503) {
          return route.fulfill({
            status,
            body: '{"detail":"Save unavailable"}',
            contentType: 'application/json',
          })
        }
      }
      await route.continue()
    })
    await page.getByLabel('Your native language', { exact: true }).selectOption('en')
    const save = page.getByRole('button', { name: 'Save', exact: true })
    await save.click()
    await expect.poll(() => writes).toBe(1)
    await expect(save).toBeDisabled()
    await expect(page.getByLabel('Your native language', { exact: true })).toBeDisabled()
    await expect(page.getByRole('checkbox')).toBeDisabled()
    await expect(page.getByRole('button', { name: 'Settings', exact: true })).toBeDisabled()
    await page.locator('form').evaluate((form) => {
      form.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }))
    })
    expect(writes).toBe(1)
    release()
    if (status === 503) {
      await expect(page.getByRole('alert')).toContainText('Save unavailable')
      await expect(save).toBeEnabled()
      await expect(page.getByLabel('Your native language', { exact: true })).toHaveValue('en')
      await save.click()
    }
    await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
    expect(writes).toBe(status === 200 ? 1 : 2)
    expect(await (await request.get('/api/v1/settings')).json()).toEqual({
      native_language: 'en',
      ui_language_mode: 'learning',
    })
    await page.reload()
    await page.getByRole('button', { name: 'Settings', exact: true }).click()
    await expect(page.getByLabel('Your native language', { exact: true })).toHaveValue('en')
  })
}
