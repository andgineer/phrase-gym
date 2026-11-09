import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { card, extraPackage, texts } from './extra-package'
import { expect, test } from './fixtures'
import { importPackage, playedMedia, savedTakes } from './helpers'

async function finishClip(page: Page) {
  await page.evaluate(() => {
    const media = (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia.at(-1)!
    media.dispatchEvent(new Event('ended'))
  })
}

test.beforeEach(async ({ page }) => {
  await installFakeMedia(page)
  await page.goto('/')
  await importPackage(page, extraPackage, 3)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
})

test('front and back stops play reference then the full take, with collapsed saved history', async ({
  page,
}) => {
  const history = card(page).getByRole('button', { name: 'Recording history' })
  await expect(history).toHaveCount(0)
  for (const index of [0, 1]) {
    const before = (await playedMedia(page)).length
    await card(page).getByRole('button', { name: 'Record a take' }).click()
    await card(page).getByRole('button', { name: 'Stop recording' }).click()
    await expect(page.getByTestId('practice-phrase')).toHaveText(texts[0])
    await expect
      .poll(async () => (await playedMedia(page)).slice(before))
      .toEqual([expect.stringMatching(/\/reference$/)])
    await finishClip(page)
    await expect
      .poll(async () => (await playedMedia(page)).slice(before))
      .toEqual([expect.stringMatching(/\/reference$/), expect.stringMatching(/^blob:/)])
    await finishClip(page)
    await expect(card(page).getByRole('button', { name: 'Record a take' })).toBeVisible()
    await expect.poll(async () => (await savedTakes(page, texts[0])).length).toBe(index + 1)
    if (index === 0) {
      await expect(history).toHaveAttribute('aria-expanded', 'false')
      await expect(card(page).locator('.take')).toHaveCount(0)
      await history.click()
    }
    await expect(card(page).locator('.take')).toHaveCount(index + 1)
  }
  const historyTake = card(page).locator('.take').first()
  const [latest] = await savedTakes(page, texts[0])
  const before = (await playedMedia(page)).length
  await expect(historyTake.locator('time')).toHaveText(latest.learning_date)
  await expect(historyTake.getByRole('button')).toHaveCount(1)
  await expect(historyTake).not.toContainText('Cue:')
  await expect(historyTake).not.toContainText('Answer then:')
  await historyTake.getByRole('button', { name: 'Mine' }).click()
  await expect
    .poll(async () => (await playedMedia(page)).slice(before))
    .toEqual([expect.stringContaining(latest.url)])
  await finishClip(page)
  await page.reload()
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(history).toHaveAttribute('aria-expanded', 'false')
  await history.click()
  await expect(card(page).locator('.take')).toHaveCount(2)
  await card(page).getByRole('button', { name: 'Good', exact: true }).click()
  await expect(history).toHaveCount(0)
  await card(page).getByRole('button', { name: 'Show', exact: true }).click()
  await expect(history).toHaveAttribute('aria-expanded', 'false')
  await history.click()
  await expect(card(page).getByText('No saved recordings yet.')).toBeVisible()
})

test('Ref before a back take replays only that take; later takes and cards compare reference first', async ({
  page,
}) => {
  const reference = card(page).getByRole('button', { name: 'Ref' })
  const microphone = card(page).getByRole('button', { name: 'Record a take' })
  const playedSince = async (before: number) => (await playedMedia(page)).slice(before)
  async function recordTake() {
    const before = (await playedMedia(page)).length
    await microphone.click()
    await card(page).getByRole('button', { name: 'Stop recording' }).click()
    return before
  }
  async function expectReferenceFirst(before: number) {
    await expect.poll(() => playedSince(before)).toEqual([expect.stringMatching(/\/reference$/)])
    await finishClip(page)
    await expect
      .poll(() => playedSince(before))
      .toEqual([expect.stringMatching(/\/reference$/), expect.stringMatching(/^blob:/)])
    await finishClip(page)
    await expect(microphone).toBeVisible()
  }

  await card(page).getByRole('button', { name: 'Show', exact: true }).click()
  await reference.click()
  await expect.poll(async () => (await playedMedia(page)).at(-1)).toMatch(/\/reference$/)
  await finishClip(page)
  const afterRef = await recordTake()
  await expect.poll(() => playedSince(afterRef)).toEqual([expect.stringMatching(/^blob:/)])
  await finishClip(page)
  await expect(microphone).toBeVisible()
  expect(await playedSince(afterRef)).toEqual([expect.stringMatching(/^blob:/)])

  await expectReferenceFirst(await recordTake())

  await reference.click()
  await expect.poll(async () => (await playedMedia(page)).at(-1)).toMatch(/\/reference$/)
  await card(page).getByRole('button', { name: 'Good', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await expectReferenceFirst(await recordTake())
  await expect.poll(async () => (await savedTakes(page, texts[0])).length).toBe(2)
})

test('phone audio controls keep playback, microphone and expandable history together', async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 320, height: 740 })
  const controls = card(page).getByRole('group', { name: 'Audio controls' })
  await controls.getByRole('button', { name: 'Record a take' }).click()
  await controls.getByRole('button', { name: 'Stop recording' }).click()
  await expect.poll(async () => (await playedMedia(page)).length).toBe(1)
  await finishClip(page)
  await expect.poll(async () => (await playedMedia(page)).length).toBe(2)
  await finishClip(page)
  await expect.poll(async () => (await savedTakes(page, texts[0])).length).toBe(1)

  const history = controls.getByRole('button', { name: 'Recording history' })
  await expect(history).toHaveText('▾')
  const reference = controls.getByRole('button', { name: 'Ref' })
  const microphone = controls.getByRole('button', { name: 'Record a take' })
  const mine = controls.getByRole('button', { name: 'Mine' })
  for (const width of [320, 390]) {
    await page.setViewportSize({ width, height: 844 })
    const boxes = []
    for (const button of [reference, microphone, mine, history]) {
      await expect(button).toBeVisible()
      const box = (await button.boundingBox())!
      expect(box.x).toBeGreaterThanOrEqual(0)
      expect(box.x + box.width).toBeLessThanOrEqual(width)
      expect(box.height).toBeGreaterThanOrEqual(44)
      boxes.push(box)
    }
    expect(boxes[0].x + boxes[0].width).toBeLessThanOrEqual(boxes[1].x)
    expect(boxes[1].x + boxes[1].width).toBeLessThanOrEqual(boxes[2].x)
    expect(Math.abs(boxes[0].y - boxes[2].y)).toBeLessThan(1)
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBe(width)
  }
  await page.screenshot({ path: testInfo.outputPath('phone-audio-controls.png') })
  await reference.click()
  await expect.poll(async () => (await playedMedia(page)).at(-1)).toMatch(/\/reference$/)
  await finishClip(page)
  await mine.click()
  await expect.poll(async () => (await playedMedia(page)).at(-1)).toMatch(/^blob:/)
  await finishClip(page)
  await history.click()
  await expect(history).toHaveText('▴')
  await expect(history).toHaveAttribute('aria-expanded', 'true')
  const recordings = controls.getByRole('region', { name: 'Saved recordings' })
  await expect(recordings.locator('.take')).toHaveCount(1)
  await expect(recordings.getByRole('heading')).toHaveCount(0)
  await expect(recordings.getByRole('button', { name: 'Compare' })).toHaveCount(0)
  expect(await microphone.evaluate((button) => getComputedStyle(button).borderRadius)).toBe(
    await reference.evaluate((button) => getComputedStyle(button).borderRadius),
  )
  expect(await recordings.evaluate((panel) => getComputedStyle(panel).borderTopStyle)).toBe('solid')
  const historyBox = (await history.boundingBox())!
  expect((await recordings.boundingBox())!.y).toBeGreaterThanOrEqual(
    historyBox.y + historyBox.height,
  )
  await history.click()
  await expect(history).toHaveText('▾')
  await expect(history).toHaveAttribute('aria-expanded', 'false')
  await expect(recordings).toHaveCount(0)
})

test('failed automatic reveal retains the take and lets Show retry without playing a hidden answer', async ({
  page,
}) => {
  await page.route('**/api/practice/reveal', (route) =>
    route.fulfill({ status: 503, json: { detail: 'Reveal temporarily unavailable' } }),
  )
  await card(page).getByRole('button', { name: 'Record a take' }).click()
  await card(page).getByRole('button', { name: 'Stop recording' }).click()
  await expect(page.getByRole('alert')).toContainText('Reveal temporarily unavailable')
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await expect.poll(async () => (await savedTakes(page, texts[0])).length).toBe(1)
  expect(await playedMedia(page)).toEqual([])
  await page.unroute('**/api/practice/reveal')
  await card(page).getByRole('button', { name: 'Show', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText(texts[0])
  await expect(card(page).getByRole('button', { name: 'Mine' })).toBeVisible()
})

test('leaving during automatic reveal prevents late playback', async ({ page }) => {
  let releaseReveal!: () => void
  const held = new Promise<void>((resolve) => (releaseReveal = resolve))
  let revealing = false
  await page.route('**/api/practice/reveal', async (route) => {
    revealing = true
    await held
    await route.continue()
  })
  await card(page).getByRole('button', { name: 'Record a take' }).click()
  await card(page).getByRole('button', { name: 'Stop recording' }).click()
  await expect.poll(() => revealing).toBe(true)
  await page.getByRole('navigation').getByRole('button', { name: 'Library', exact: true }).click()
  releaseReveal()
  await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
  expect(await playedMedia(page)).toEqual([])
  await expect.poll(async () => (await savedTakes(page, texts[0])).length).toBe(1)
})
