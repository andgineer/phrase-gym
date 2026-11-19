import type { Page } from '@playwright/test'
import { expect, test } from './fixtures'
import { extraPackage } from './extra-package'
import { importPackage, revealAnswer } from './helpers'

test.use({
  viewport: { width: 390, height: 844 },
  hasTouch: true,
  isMobile: true,
  coachEnabled: false,
})
const tab = (page: Page, name: string) =>
  page.getByRole('navigation').getByRole('button', { name, exact: true })

async function color(page: Page, selector: string, property = 'backgroundColor') {
  return page
    .locator(selector)
    .first()
    .evaluate((element, name) => {
      return getComputedStyle(element)[name as 'backgroundColor']
    }, property)
}

test('built client exposes installation metadata, icons and the Coach shortcut', async ({
  page,
}) => {
  await page.goto('/')
  const manifestLink = page.locator('link[rel="manifest"]')
  await expect(manifestLink).toHaveAttribute('href', '/manifest.webmanifest')
  const response = await page.request.get((await manifestLink.getAttribute('href'))!)
  expect(response.headers()['content-type']).toBe('application/manifest+json')
  const manifest = await response.json()
  expect(manifest).toMatchObject({
    name: 'Phrase Gym',
    short_name: 'Phrase Gym',
    start_url: '/',
    display: 'standalone',
    theme_color: '#f7f4ec',
    background_color: '#f7f4ec',
  })
  for (const icon of manifest.icons) {
    const dimensions = await page.evaluate(async (url) => {
      const image = new Image()
      image.src = url
      await image.decode()
      return `${image.naturalWidth}x${image.naturalHeight}`
    }, icon.src)
    expect(dimensions).toBe(icon.sizes)
  }
  expect(manifest.icons.map((icon: { purpose: string }) => icon.purpose)).toContain('maskable')
  await expect(page.locator('meta[name="viewport"]')).toHaveAttribute(
    'content',
    /viewport-fit=cover/,
  )
  await expect(page.locator('meta[name="apple-mobile-web-app-capable"]')).toHaveAttribute(
    'content',
    'yes',
  )
  await expect(page.locator('meta[name="apple-mobile-web-app-title"]')).toHaveAttribute(
    'content',
    'Phrase Gym',
  )
  const apple = page.locator('link[rel="apple-touch-icon"]')
  await expect(apple).toHaveAttribute('sizes', '180x180')
  expect((await page.request.get((await apple.getAttribute('href'))!)).ok()).toBe(true)
  expect(
    (await page.request.get((await page.locator('link[rel="icon"]').getAttribute('href'))!)).ok(),
  ).toBe(true)
  await page.goto(manifest.shortcuts.find((item: { name: string }) => item.name === 'Coach').url)
  await expect(tab(page, 'Coach')).toHaveAttribute('aria-current', 'page')
  expect(
    await page.evaluate(() =>
      navigator.serviceWorker.getRegistrations().then((items) => items.length),
    ),
  ).toBe(0)
})

test('safe areas reserve shell, scrolling content and dock space; fields avoid focus zoom', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 3)
  for (const name of ['Practice', 'Library', 'Coach', 'Settings']) {
    await tab(page, name).tap()
    await expect(tab(page, name)).toHaveAttribute('aria-current', 'page')
    await expect(page.locator('input, textarea, select').first()).toBeVisible()
    const sizes = await page
      .locator('input:not([type="radio"]):not([type="checkbox"]), textarea, select')
      .evaluateAll((fields) =>
        fields.map((field) => Number.parseFloat(getComputedStyle(field).fontSize)),
      )
    expect(sizes.length).toBeGreaterThan(0)
    for (const size of sizes) expect(size).toBeGreaterThanOrEqual(16)
  }
  await tab(page, 'Practice').tap()
  await page.getByRole('button', { name: 'Start', exact: true }).tap()
  await revealAnswer(page, true)
  const originalMaxHeight = await page
    .locator('.practice-card')
    .evaluate((element) => Number.parseFloat(getComputedStyle(element).maxHeight))
  // Desktop engines report zero env() insets; inject their CSS values to exercise the layout.
  await page.addStyleTag({
    content:
      ':root { --safe-top: 47px; --safe-bottom: 34px; --safe-left: 10px; --safe-right: 14px; }',
  })
  const shell = await page.locator('.shell').evaluate((element) => {
    const style = getComputedStyle(element)
    return [style.paddingTop, style.paddingLeft, style.paddingRight]
  })
  expect(shell).toEqual(['65px', '26px', '30px'])
  const card = await page.locator('.practice-card').boundingBox()
  const dock = await page.locator('.rating-dock').boundingBox()
  expect(
    await page
      .locator('.practice-card')
      .evaluate((element) => Number.parseFloat(getComputedStyle(element).maxHeight)),
  ).toBeCloseTo(originalMaxHeight - 81)
  expect(card!.height).toBeLessThanOrEqual(originalMaxHeight - 81)
  expect(card!.y + card!.height).toBeLessThanOrEqual(dock!.y)
  expect(
    await page
      .locator('.rating-dock')
      .evaluate((element) => getComputedStyle(element).paddingBottom),
  ).toBe('46px')
  const skip = await page.getByRole('button', { name: 'Skip', exact: true }).boundingBox()
  expect(skip!.y + skip!.height).toBeLessThanOrEqual(844 - 34)
  expect(dock!.x).toBe(26)
  expect(dock!.x + dock!.width).toBe(390 - 30)
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390)
})

test('dark palette covers pages, card controls, counts, progress and history', async ({
  page,
}, testInfo) => {
  await page.emulateMedia({ colorScheme: 'dark' })
  await page.goto('/')
  await importPackage(page, extraPackage, 3)
  expect(await color(page, '.waiting-counts', 'color')).toBe('rgb(233, 238, 233)')
  expect(await color(page, '.lesson-combo input')).toBe('rgb(48, 67, 57)')
  for (const name of ['Library', 'Coach', 'Settings', 'Practice']) {
    await tab(page, name).tap()
    expect(await color(page, '.page-tabs')).toBe('rgb(30, 45, 38)')
    expect(await color(page, '[aria-current="page"]')).toBe('rgb(37, 53, 46)')
    await expect(page.locator('input, textarea, select').first()).toBeVisible()
    for (const field of await page
      .locator('input[type="text"], input[type="number"], input[type="search"], textarea, select')
      .all()) {
      expect(['rgb(233, 238, 233)', 'rgb(196, 239, 209)']).toContain(
        await field.evaluate((element) => getComputedStyle(element).color),
      )
      expect(await field.evaluate((element) => getComputedStyle(element).backgroundColor)).not.toBe(
        'rgb(255, 255, 255)',
      )
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390)
  }
  await page.getByRole('button', { name: 'Start', exact: true }).tap()
  expect(await color(page, '.tinted')).toBe('rgb(48, 67, 57)')
  await page.getByRole('button', { name: 'Hint', exact: true }).tap()
  expect(await color(page, '.hint', 'color')).toBe('rgb(165, 209, 184)')
  expect(await color(page, '.practice-progress')).toBe('rgb(64, 87, 74)')
  await revealAnswer(page, true)
  await page.getByRole('button', { name: 'More actions' }).tap()
  await page.getByRole('button', { name: 'Recording history' }).tap()
  expect(await color(page, '.recording-history-panel', 'borderTopColor')).toBe('rgb(100, 119, 107)')
  await page.getByRole('button', { name: 'Close recording history' }).tap()
  expect(await color(page, '.rating-dock')).toBe('rgb(24, 34, 31)')
  expect(await color(page, '.not-yet')).toBe('rgb(82, 44, 41)')
  expect(await color(page, '.effort')).toBe('rgb(73, 60, 27)')
  expect(await color(page, '.recalled')).toBe('rgb(36, 73, 50)')
  await page.getByRole('button', { name: 'More actions' }).tap()
  expect(await color(page, '.card-menu-panel')).toBe('rgb(37, 53, 46)')
  await page.keyboard.press('Escape')
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(390)
  await page.screenshot({ path: testInfo.outputPath('home-screen-dark.png'), fullPage: true })
})
