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
  await historyTake.getByRole('button', { name: 'Compare' }).click()
  await expect
    .poll(async () => (await playedMedia(page)).slice(before))
    .toEqual([expect.stringMatching(/\/reference$/)])
  await finishClip(page)
  await expect
    .poll(async () => (await playedMedia(page)).slice(before))
    .toEqual([expect.stringMatching(/\/reference$/), expect.stringContaining(latest.url)])
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
  await expect(card(page).getByRole('button', { name: '▶ Mine' })).toBeVisible()
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
