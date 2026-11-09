import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { extraPackage, modeLine } from './extra-package'
import { importPackage, rateNew } from './helpers'

const status = (page: Page, text: string) => page.getByRole('status').filter({ hasText: text })

test.beforeEach(async ({ page }) => installFakeMedia(page))

test('daily limits are edited in Settings and govern Daily', async ({ page }) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 3)
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  const due = page.getByLabel('Due phrases per day')
  const fresh = page.getByLabel('New phrases per day')
  const save = page.getByRole('button', { name: 'Save settings' })
  await expect(due).toHaveValue('20')
  await expect(fresh).toHaveValue('5')
  await due.fill('0')
  await fresh.fill('0')
  await save.click()
  await expect(status(page, 'cannot both be zero')).toBeVisible()
  await fresh.fill('2')
  await save.click()
  await expect(status(page, 'Settings saved.')).toBeVisible()

  await page.getByRole('button', { name: 'Practice', exact: true }).click()
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Daily · 1 / 2 new')
  await rateNew(page)
  await expect(modeLine(page)).toHaveText('Daily · 2 / 2 new')
  await rateNew(page)
  await expect(page.getByText("Today's Daily practice is complete.", { exact: true })).toBeVisible()

  await page.reload()
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await expect(due).toHaveValue('0')
  await expect(fresh).toHaveValue('2')
})

test.describe('mobile', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true })

  test('mobile Settings limits fit the screen', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('button', { name: 'Settings', exact: true }).tap()
    await expect(page.getByLabel('New phrases per day')).toBeVisible()
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
  })
})
