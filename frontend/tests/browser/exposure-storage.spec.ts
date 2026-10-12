import { expect } from '@playwright/test'
import { test } from './fixtures'

test('exposure storage failure prevents reveal and can recover explicitly', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.addInitScript(() => {
    let failing = true
    const original = Storage.prototype.setItem
    Storage.prototype.setItem = function (key, value) {
      if (failing && key.startsWith('phrase-gym.reference-exposure.v1.'))
        throw new DOMException('Reference event storage is full', 'QuotaExceededError')
      original.call(this, key, value)
    }
    Object.defineProperty(window, 'allowReferenceStorage', {
      value: () => {
        failing = false
      },
    })
  })
  let exposures = 0
  await page.route('**/api/v1/exposures', async (route) => {
    exposures += 1
    await route.continue()
  })
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await expect(page.getByRole('alert')).toContainText('Reference event storage is full')
  await expect(page.locator('.phrase-card')).toHaveCount(0)
  expect(exposures).toBe(0)
  await page.evaluate(() =>
    (window as unknown as { allowReferenceStorage: () => void }).allowReferenceStorage(),
  )
  await page.getByRole('alert').getByRole('button', { name: 'Retry' }).click()
  await expect(page.locator('.phrase-card')).toBeVisible()
  await expect.poll(() => exposures).toBe(1)
  await expect(page.getByRole('alert')).toHaveCount(0)
})
