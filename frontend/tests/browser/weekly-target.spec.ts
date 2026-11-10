import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { extraPackage } from './extra-package'
import { importPackage, rateNew } from './helpers'

// Wednesday 12:00 in Europe/Belgrade; its week runs Monday 2026-11-02 to Sunday 2026-11-08.
const start = Date.parse('2026-11-04T11:00:00Z')
const fact = (page: Page, id: string) => page.getByTestId(id)
const target = (page: Page) => page.getByLabel('Practice days per week')
const pause = (page: Page) => page.getByRole('checkbox', { name: 'Pause weekly target' })

async function setClock(page: Page, days: number) {
  const instant = new Date(start + days * 24 * 3600 * 1000)
  await page.request.post(`/__test/clock/${instant.toISOString()}`)
  await page.clock.setFixedTime(instant)
}

async function openSettings(page: Page) {
  await page.getByRole('navigation').getByRole('button', { name: 'Settings', exact: true }).click()
  await expect(target(page)).toBeVisible()
}

async function saveAndReturn(page: Page) {
  await page.getByRole('button', { name: 'Save settings' }).click()
  await expect(page.getByRole('status').filter({ hasText: 'Settings saved.' })).toBeVisible()
  await page.getByRole('navigation').getByRole('button', { name: 'Practice', exact: true }).click()
}

async function practiseOnce(page: Page) {
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await rateNew(page)
  await page.getByRole('navigation').getByRole('button', { name: 'Practice', exact: true }).click()
}

test.beforeEach(async ({ page }) => {
  await installFakeMedia(page)
  await setClock(page, 0)
})

test('a chosen weekly target shows this week’s days and can be changed, paused and resumed', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 3)
  await expect(fact(page, 'week')).toHaveCount(0)

  await openSettings(page)
  await expect(target(page).locator('option:checked')).toHaveText('No target')
  await expect(pause(page)).toHaveCount(0)
  await target(page).selectOption('3')
  await expect(pause(page)).not.toBeChecked()
  await saveAndReturn(page)
  await expect(fact(page, 'week')).toHaveText('This week: 0 of 3 days')
  await expect(fact(page, 'streak')).toHaveCount(0)
  const daily = await page.getByRole('button', { name: 'Daily', exact: true }).boundingBox()
  const week = await fact(page, 'week').boundingBox()
  expect(daily!.y).toBeLessThan(week!.y)

  await practiseOnce(page)
  await expect(fact(page, 'week')).toHaveText('This week: 1 of 3 days')
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')

  await openSettings(page)
  await target(page).selectOption('2')
  await saveAndReturn(page)
  await expect(fact(page, 'week')).toHaveText('This week: 1 of 2 days')

  // Thursday is missed; Friday's practice still reaches the target.
  await setClock(page, 2)
  await page.reload()
  await expect(fact(page, 'streak')).toHaveText('No current practice streak')
  await expect(fact(page, 'week')).toHaveText('This week: 1 of 2 days')
  await practiseOnce(page)
  await expect(fact(page, 'week')).toHaveText('This week: 2 days · target reached')

  await openSettings(page)
  await pause(page).check()
  await saveAndReturn(page)
  await expect(fact(page, 'week')).toHaveCount(0)
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')

  await page.reload()
  await openSettings(page)
  await expect(target(page)).toHaveValue('2')
  await expect(pause(page)).toBeChecked()
  await pause(page).uncheck()
  await saveAndReturn(page)
  await expect(fact(page, 'week')).toHaveText('This week: 2 days · target reached')

  // The whole next week passes without practice; the return owes nothing.
  await setClock(page, 14)
  await page.reload()
  await expect(fact(page, 'week')).toHaveText('This week: 0 of 2 days')
  await expect(fact(page, 'best-streak')).toHaveText('Personal best: 1 day')
  await expect(fact(page, 'progress-title')).toHaveText('Progress — 2 days')

  await openSettings(page)
  await target(page).selectOption({ label: 'No target' })
  await expect(pause(page)).toHaveCount(0)
  await saveAndReturn(page)
  await expect(fact(page, 'week')).toHaveCount(0)
})

test.describe('mobile', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true })

  test('the weekly target fits a phone in Settings and on entry', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('navigation').getByRole('button', { name: 'Settings', exact: true }).tap()
    await target(page).selectOption('4')
    await page.getByRole('button', { name: 'Save settings' }).tap()
    await expect(page.getByRole('status').filter({ hasText: 'Settings saved.' })).toBeVisible()
    await expect(pause(page)).toBeVisible()
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
    await page.getByRole('navigation').getByRole('button', { name: 'Practice', exact: true }).tap()
    await expect(fact(page, 'week')).toHaveText('This week: 0 of 4 days')
    await expect(page.getByRole('button', { name: 'Daily', exact: true })).toBeInViewport()
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
  })
})
