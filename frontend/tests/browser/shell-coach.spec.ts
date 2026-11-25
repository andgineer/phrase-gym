import { expect, test } from './fixtures'
import { extraPackage, card } from './extra-package'
import { importPackage, phraseSearch, revealAnswer } from './helpers'

test.use({ viewport: { width: 390, height: 844 }, coachEnabled: false })

const tab = (page: import('@playwright/test').Page, name: string) =>
  page.getByRole('navigation').getByRole('button', { name, exact: true })

test('phone pages, card sides, version and hash stay in the retained shell', async ({
  page,
}, testInfo) => {
  await page.goto('/#coach')
  const version = (await (await page.request.get('/api/health')).json()).version
  const history = await page.evaluate(() => window.history.length)
  for (const name of ['Coach', 'Library', 'Settings', 'Practice']) {
    await tab(page, name).click()
    await expect(tab(page, name)).toHaveAttribute('aria-current', 'page')
    await expect(page.locator('h1.visually-hidden')).toHaveText(name)
    await expect(page.locator('.app-version')).toHaveText(`v${version}`)
    if (name === 'Coach')
      await page.screenshot({
        path: `/tmp/phrase-gym-shell-coach-${testInfo.project.name}.png`,
        fullPage: true,
      })
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
    const tabs = await tab(page, name).boundingBox()
    expect(tabs!.height).toBeGreaterThanOrEqual(44)
    expect(await page.evaluate(() => window.history.length)).toBe(history)
    await page.reload()
    await expect(tab(page, name)).toHaveAttribute('aria-current', 'page')
  }
  await importPackage(page, extraPackage, 3)
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  await expect(card(page)).toBeVisible()
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
  await revealAnswer(page)
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
  await page.getByRole('link', { name: 'Phrase Gym' }).click()
  await expect(card(page)).toHaveCount(0)
  await expect(tab(page, 'Practice')).toHaveAttribute('aria-current', 'page')
})

test('Coach normalizes pasted lines, Enter sends, and errors preserve the draft', async ({
  page,
}) => {
  await page.goto('/#coach')
  const field = page.getByRole('textbox', { name: 'Phrase or word for Coach' })
  const send = page.getByRole('button', { name: 'Send', exact: true })
  await expect(send).toBeDisabled()
  await field.fill('  by and large\nThe plan worked.  ')
  const request = page.waitForRequest('**/api/coach/requests')
  await field.press('Enter')
  expect((await request).postDataJSON()).toEqual({
    text: 'by and large The plan worked.',
    learning_language: 'en',
  })
  await expect(page.getByRole('status')).toContainText('Saved for Coach.')
  await expect(field).toHaveValue('')
  await page.route('**/api/coach/requests', (route) =>
    route.fulfill({ status: 503, json: { detail: 'Try again later' } }),
  )
  await field.fill('keep this draft')
  await send.click()
  await expect(page.getByRole('alert')).toHaveText('Try again later')
  await expect(field).toHaveValue('keep this draft')
  await expect(send).toBeEnabled()
})

test('Library offers Coach after an empty search and after matching results', async ({ page }) => {
  await page.goto('/')
  await page.getByRole('link', { name: 'Coach', exact: true }).click()
  await expect(tab(page, 'Coach')).toHaveAttribute('aria-current', 'page')
  await importPackage(page, extraPackage, 3)
  await tab(page, 'Library').click()
  await phraseSearch(page).fill('  a new expression  ')
  await expect(page.getByText('Not in your Library yet.', { exact: true })).toBeVisible()
  const button = page.getByRole('button', { name: /Send “a new expression” to Coach/ })
  const request = page.waitForRequest('**/api/coach/requests')
  await button.click()
  expect((await request).postDataJSON()).toEqual({
    text: 'a new expression',
    learning_language: 'en',
  })
  await expect(page.locator('.library-coach')).toHaveText('Saved for Coach.')
  await phraseSearch(page).fill('good')
  await expect(page.getByRole('list', { name: 'Phrases' })).toBeVisible()
  const quiet = page.getByRole('button', { name: 'Not it? Send “good” to Coach' })
  await quiet.click()
  await expect(page.locator('.library-coach')).toHaveText('Saved for Coach.')
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
})
