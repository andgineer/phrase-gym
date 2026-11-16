import { expect, test } from './fixtures'
import { extraPackage } from './extra-package'
import { importPackage, rateNew } from './helpers'
import { installFakeMedia } from './fake-media'

test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true })
test.beforeEach(async ({ page }) => installFakeMedia(page))

test('phone card keeps controls still across reveal, hint, history and explanation', async ({
  page,
}, testInfo) => {
  await page.goto('/')
  const value = structuredClone(extraPackage)
  for (const phrase of value.lessons[0]!.phrases)
    phrase.detailed_explanation = 'Longer explanation. '.repeat(80)
  await importPackage(page, value, 3)
  await page.getByRole('button', { name: 'Start', exact: true }).tap()
  const mic = page.getByRole('button', { name: 'Record a take' })
  const fixedBounds = () =>
    page
      .locator(
        '.card-controls > button, .practice-card > .microphone, .rating-dock button, .card-menu > button',
      )
      .evaluateAll((controls) =>
        controls.map((control) => {
          const r = control.getBoundingClientRect()
          return { x: r.x, y: r.y, width: r.width, height: r.height }
        }),
      )
  const frontBounds = await fixedBounds()
  const front = await mic.boundingBox()
  const row = page.locator('.card-controls')
  const controls = await row.boundingBox()
  await page.getByRole('button', { name: 'Hint', exact: true }).tap()
  await expect(page.locator('.hint')).toBeVisible()
  expect(await mic.boundingBox()).toEqual(front)
  expect(await row.boundingBox()).toEqual(controls)
  expect(await fixedBounds()).toEqual(frontBounds)
  await page.getByRole('button', { name: 'Reveal', exact: true }).tap()
  await expect(page.getByTestId('practice-phrase')).toBeVisible()
  expect(await mic.boundingBox()).toEqual(front)
  const backControls = await row.boundingBox()
  const backBounds = await fixedBounds()
  const ratings = page.locator('.rating-dock')
  const dock = await ratings.boundingBox()
  await page.getByRole('button', { name: 'Recording history' }).tap()
  await expect(page.getByRole('region', { name: 'Saved recordings' })).toBeVisible()
  expect(await mic.boundingBox()).toEqual(front)
  expect(await row.boundingBox()).toEqual(backControls)
  expect(await fixedBounds()).toEqual(backBounds)
  expect(await ratings.boundingBox()).toEqual(dock)
  await page.getByRole('button', { name: 'Recording history' }).tap()
  const explanation = page.getByRole('button', { name: 'More explanation' })
  const link = await explanation.boundingBox()
  await explanation.tap()
  expect(await explanation.boundingBox()).toEqual(link)
  expect(await mic.boundingBox()).toEqual(front)
  expect(await row.boundingBox()).toEqual(backControls)
  expect(await fixedBounds()).toEqual(backBounds)
  expect(await ratings.boundingBox()).toEqual(dock)
  const content = page.locator('.card-content')
  expect(await content.evaluate((element) => element.scrollHeight > element.clientHeight)).toBe(
    true,
  )
  await content.evaluate((element) => {
    element.scrollTop = 100
  })
  expect(await fixedBounds()).toEqual(backBounds)
  await expect(page.getByRole('button', { name: 'Good', exact: true })).toBeInViewport()
  await page.getByRole('button', { name: 'More actions' }).tap()
  await expect(page.getByRole('button', { name: 'Delete phrase…' })).toBeVisible()
  await page.keyboard.press('Escape')
  await expect(page.getByRole('button', { name: 'Delete phrase…' })).toHaveCount(0)
  await page.getByRole('button', { name: 'More actions' }).tap()
  await page.getByTestId('mode-line').tap()
  await expect(page.getByRole('button', { name: 'Delete phrase…' })).toHaveCount(0)
  await content.evaluate((element) => {
    element.scrollTop = 0
  })
  expect(await page.evaluate(() => document.documentElement.scrollHeight)).toBeLessThanOrEqual(844)
  await page.screenshot({ path: testInfo.outputPath('practice-back.png'), fullPage: true })
})

test('Start and Extra reflect real remaining allowances', async ({ page }) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 3)
  const settings = await (await page.request.get('/api/settings')).json()
  await page.request.put('/api/settings', { data: { ...settings, daily_new_limit: 1 } })
  await page.reload()
  const start = page.getByRole('button', { name: 'Start', exact: true })
  const extra = page.getByRole('button', { name: 'Extra', exact: true })
  await expect(start).toBeEnabled()
  await expect(extra).toHaveCount(0)
  await expect(page.locator('.waiting-counts')).toHaveText('0 due1 new')
  await start.tap()
  await rateNew(page, true)
  await expect(start).toBeDisabled()
  await expect(extra).toBeEnabled()
  await extra.tap()
  await rateNew(page, true)
  await extra.tap()
  await rateNew(page, true)
  await expect(start).toBeDisabled()
  await expect(extra).toHaveCount(0)
})
