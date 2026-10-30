import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { card, details, extraPackage, modeLine, openDetails, texts } from './extra-package'
import { importPackage } from './helpers'

const press = (page: Page, name: string) => page.getByRole('button', { name, exact: true }).click()

// The server commits the press, but the browser never sees its response.
async function loseFirstExtraResponse(page: Page): Promise<string[]> {
  const operations: string[] = []
  await page.route('**/api/practice/extra', async (route) => {
    operations.push(route.request().postDataJSON().operation_id)
    if (operations.length > 1) return route.continue()
    await route.fetch()
    await route.abort()
  })
  return operations
}

test.beforeEach(async ({ page }) => installFakeMedia(page))

test('failed Extra after Stop stays on entry with a working Retry that reuses the operation ID', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 'Added 3 phrases.')
  await press(page, 'Extra')
  await expect(modeLine(page)).toHaveText('Extra · 1 / 5 new')
  await press(page, 'Stop')
  await expect(card(page)).toHaveCount(0)
  const operations = await loseFirstExtraResponse(page)
  await press(page, 'Extra')
  await expect(page.getByRole('alert')).toContainText('Extra was not confirmed; retry it.')
  await expect(card(page)).toHaveCount(0)
  await press(page, 'Retry')
  await expect(modeLine(page)).toHaveText('Extra · 1 / 5 new')
  await expect(card(page).getByRole('heading', { name: texts[0] })).toBeVisible()
  await expect(page.getByRole('alert')).toHaveCount(0)
  expect(operations).toHaveLength(2)
  expect(operations[1]).toBe(operations[0])
  await page.getByRole('button', { name: 'Next' }).click()
  await expect(modeLine(page)).toHaveText('Extra · 2 / 5 new')
})

test('failed Practice open after leaving a Practice card stays on details with an error', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 'Added 3 phrases.')
  await openDetails(page, texts[0])
  await details(page).getByRole('button', { name: 'Practice' }).click()
  await expect(modeLine(page)).toHaveText('Practice')
  await press(page, 'Practice')
  await expect(card(page)).toHaveCount(0)
  await openDetails(page, texts[1])
  await page.route('**/api/practice/phrase', (route) => route.abort())
  await details(page).getByRole('button', { name: 'Practice' }).click()
  await expect(page.getByRole('alert')).toBeVisible()
  await expect(card(page)).toHaveCount(0)
  await expect(details(page).getByRole('heading', { name: texts[1] })).toBeVisible()
  await page.unroute('**/api/practice/phrase')
  await details(page).getByRole('button', { name: 'Practice' }).click()
  await expect(modeLine(page)).toHaveText('Practice')
  await expect(card(page).getByRole('heading', { name: texts[1] })).toBeVisible()
})

test('tab A Extra retry after tab B opened Practice leaves tab B Practice completable', async ({
  page,
  context,
}) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 'Added 3 phrases.')
  const operations = await loseFirstExtraResponse(page)
  await press(page, 'Extra')
  await expect(page.getByRole('button', { name: 'Retry', exact: true })).toBeVisible()

  const other = await context.newPage()
  await installFakeMedia(other)
  await other.goto('/')
  await openDetails(other, texts[2])
  await details(other).getByRole('button', { name: 'Practice' }).click()
  await expect(modeLine(other)).toHaveText('Practice')

  await press(page, 'Retry')
  await expect(page.getByText('Extra was saved. Press Daily to continue.')).toBeVisible()
  await expect(card(page)).toHaveCount(0)
  expect(operations).toEqual([operations[0], operations[0]])

  await other.getByRole('button', { name: 'Next' }).click()
  await expect(details(other).getByRole('heading', { name: texts[2] })).toBeVisible()
  await expect(details(other).getByText(/^Next due: /)).toBeVisible()
  await expect(other.getByRole('alert')).toHaveCount(0)

  await press(page, 'Daily')
  await expect(modeLine(page)).toHaveText('Extra · 1 / 5 new')
})
