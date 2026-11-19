import { expect, test } from './fixtures'
import { extraPackage } from './extra-package'
import { importPackage, rateNew, revealAnswer } from './helpers'
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
  await expect(mic).toBeVisible()
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
  expect(frontBounds).toHaveLength(4)
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
  const contained = await page.locator('.back-controls button').evaluateAll((buttons) =>
    buttons.map((button) => {
      const bounds = button.getBoundingClientRect()
      const text = document.createRange()
      text.selectNodeContents(button)
      const contents = [...text.getClientRects()].filter((rect) => rect.width && rect.height)
      return {
        height: bounds.height,
        width: bounds.width,
        contained: contents.every((rect) => rect.left >= bounds.left && rect.right <= bounds.right),
      }
    }),
  )
  for (const button of contained) {
    expect(button.height).toBe(48)
    expect(button.width).toBeGreaterThanOrEqual(44)
    expect(button.contained).toBe(true)
  }
  const backControls = await row.boundingBox()
  const backBounds = await fixedBounds()
  const ratings = page.locator('.rating-dock')
  const dock = await ratings.boundingBox()
  const grade = (await ratings.getByRole('button', { name: 'Good', exact: true }).boundingBox())!
  const skip = (await ratings.getByRole('button', { name: 'Skip', exact: true }).boundingBox())!
  expect(skip.y).toBe(grade.y)
  expect(skip.height).toBe(grade.height)
  expect(skip.width).toBe(44)
  expect(skip.width).toBeLessThan(grade.width)
  expect(skip.x).toBeGreaterThan(grade.x + grade.width)
  await page.getByRole('button', { name: 'More actions' }).tap()
  await page.getByRole('button', { name: 'Recording history' }).tap()
  const history = page.getByRole('dialog', { name: 'Recording history' })
  await expect(history.getByRole('region', { name: 'Saved recordings' })).toBeVisible()
  expect(await fixedBounds()).toEqual(backBounds)
  await history.getByRole('button', { name: 'Close recording history' }).tap()
  await expect(history).toHaveCount(0)
  const explanation = page.getByRole('button', { name: 'More explanation' })
  const link = await explanation.boundingBox()
  await explanation.tap()
  expect(await explanation.boundingBox()).toEqual(link)
  expect(await mic.boundingBox()).toEqual(front)
  expect(await row.boundingBox()).toEqual(backControls)
  expect(await fixedBounds()).toEqual(backBounds)
  expect(await ratings.boundingBox()).toEqual(dock)
  const content = page.locator('.card-content')
  const contentSize = await content.evaluate(({ clientHeight, scrollHeight }) => ({
    clientHeight,
    scrollHeight,
  }))
  expect(contentSize.clientHeight).toBeGreaterThan(320)
  expect(contentSize.scrollHeight).toBeGreaterThan(contentSize.clientHeight)
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

test('Extra reopens two skipped phrases below the five-new allowance', async ({ page }) => {
  await page.goto('/')
  const value = structuredClone(extraPackage)
  value.lessons[0]!.phrases = value.lessons[0]!.phrases.slice(0, 2)
  await importPackage(page, value, 2)
  await page.getByRole('button', { name: 'Start', exact: true }).tap()
  for (let card = 0; card < 2; card++) {
    await revealAnswer(page, true)
    await page.getByRole('button', { name: 'Skip', exact: true }).tap()
    if (card === 0) await expect(page.getByTestId('mode-line')).toHaveText('2 / 5 new')
  }
  await expect(page.getByRole('button', { name: 'Start', exact: true })).toBeDisabled()
  const extra = page.getByRole('button', { name: 'Extra', exact: true })
  await expect(extra).toBeEnabled()
  expect(await (await page.request.get('/api/practice/today')).json()).toMatchObject({
    new_count: 0,
    due_count: 0,
    extra_available: true,
  })
  await extra.tap()
  await expect(page.getByTestId('mode-line')).toHaveText('Extra · 3 / 5 new')
  await expect(page.locator('.practice-prompt')).toBeVisible()
})
