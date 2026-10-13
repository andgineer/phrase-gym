import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('explicit exposure retry sends the same retained payload and acknowledges it', async ({
  page,
}) => {
  const payloads: { id: string }[] = []
  await page.route('**/api/v1/exposures', async (route) => {
    payloads.push(route.request().postDataJSON())
    if (payloads.length === 1)
      await route.fulfill({
        status: 503,
        body: '{"detail":"Exposure unavailable"}',
        contentType: 'application/json',
      })
    else await route.continue()
  })
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await expect(page.locator('.phrase-card')).toBeHidden()
  await expect(page.getByTestId('connection-required')).toBeVisible()
  const key = `phrase-gym.reference-exposure.v1.${payloads[0].id}`
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), key)).toEqual(
    payloads[0],
  )
  const acknowledgement = page.waitForResponse(
    (response) => response.url().endsWith('/api/v1/exposures') && response.status() === 200,
  )
  await page.getByTestId('connection-required').getByRole('button', { name: 'Retry' }).click()
  expect((await (await acknowledgement).json()).id).toBe(payloads[0].id)
  await expect(page.getByRole('status')).toHaveCount(0)
  await expect(page.getByRole('alert')).toHaveCount(0)
  expect(payloads).toHaveLength(2)
  expect(payloads[1]).toEqual(payloads[0])
  await expect.poll(() => page.evaluate((key) => localStorage.getItem(key), key)).toBeNull()
})

test('a previous visit acknowledgement cannot clear a later failed visit', async ({ page }) => {
  let release!: () => void
  const gate = new Promise<void>((resolve) => {
    release = resolve
  })
  const payloads: { id: string; target_id: string }[] = []
  let retry = false
  await page.route('**/api/v1/exposures', async (route) => {
    const payload = route.request().postDataJSON()
    payloads.push(payload)
    if (payloads.length === 1) {
      await gate
      await route.continue()
    } else if (!retry)
      await route.fulfill({
        status: 503,
        body: '{"detail":"Exposure unavailable"}',
        contentType: 'application/json',
      })
    else await route.continue()
  })
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await expect(page.locator('.phrase-card')).toBeVisible()
  await expect.poll(() => payloads.length).toBe(1)
  await page.getByRole('button', { name: 'Back', exact: true }).click()
  await page.getByLabel('Find a phrase', { exact: true }).fill('coffee')
  await page.getByRole('link', { name: /catch up over coffee/ }).click()
  await expect(page.getByTestId('connection-required')).toBeVisible()
  expect(payloads[1].target_id).not.toBe(payloads[0].target_id)
  const firstKey = `phrase-gym.reference-exposure.v1.${payloads[0].id}`
  const secondKey = `phrase-gym.reference-exposure.v1.${payloads[1].id}`
  release()
  await expect.poll(() => page.evaluate((key) => localStorage.getItem(key), firstKey)).toBeNull()
  expect(await page.evaluate((key) => JSON.parse(localStorage.getItem(key)!), secondKey)).toEqual(
    payloads[1],
  )
  await expect(page.getByTestId('connection-required')).toBeVisible()
  retry = true
  const acknowledgement = page.waitForResponse(
    (response) => response.url().endsWith('/api/v1/exposures') && response.status() === 200,
  )
  await page.getByTestId('connection-required').getByRole('button', { name: 'Retry' }).click()
  expect((await (await acknowledgement).json()).id).toBe(payloads[1].id)
  await expect(page.getByRole('status')).toHaveCount(0)
  await expect(page.getByRole('alert')).toHaveCount(0)
  expect(payloads[2]).toEqual(payloads[1])
  await expect.poll(() => page.evaluate((key) => localStorage.getItem(key), secondKey)).toBeNull()
})
