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
  await page.getByRole('button', { name: 'Start', exact: true }).click()
})

test('front and back stops play reference then the full take, with saved history available from the menu', async ({
  page,
}) => {
  const actions = page.getByRole('button', { name: 'More actions' })
  const history = page.getByRole('button', { name: 'Recording history', exact: true })
  const recordings = page.getByRole('dialog', { name: 'Recording history' })
  async function openHistory() {
    await expect(recordings).toHaveCount(0)
    await expect(history).toHaveCount(0)
    await actions.click()
    await history.click()
  }
  await expect(actions).toHaveCount(0)
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
    await openHistory()
    await expect(recordings.locator('.take')).toHaveCount(index + 1)
    if (index === 0)
      await recordings.getByRole('button', { name: 'Close recording history' }).click()
  }
  const historyTake = recordings.locator('.take').first()
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
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  await openHistory()
  await expect(recordings.locator('.take')).toHaveCount(2)
  await recordings.getByRole('button', { name: 'Close recording history' }).click()
  await card(page).getByRole('button', { name: 'Good', exact: true }).click()
  await expect(actions).toHaveCount(0)
  await card(page).getByRole('button', { name: 'Reveal', exact: true }).click()
  await openHistory()
  await expect(recordings.getByText('No saved recordings yet.')).toBeVisible()
})

test('Ref on a back keeps replaying only each later take on that card; the next card compares reference first', async ({
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

  await card(page).getByRole('button', { name: 'Reveal', exact: true }).click()
  await reference.click()
  await expect.poll(async () => (await playedMedia(page)).at(-1)).toMatch(/\/reference$/)
  await finishClip(page)
  for (let take = 0; take < 3; take++) {
    const afterRef = await recordTake()
    await expect.poll(() => playedSince(afterRef)).toEqual([expect.stringMatching(/^blob:/)])
    await finishClip(page)
    await expect(microphone).toBeVisible()
    expect(await playedSince(afterRef)).toEqual([expect.stringMatching(/^blob:/)])
  }

  await reference.click()
  await expect.poll(async () => (await playedMedia(page)).at(-1)).toMatch(/\/reference$/)
  await card(page).getByRole('button', { name: 'Good', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await expectReferenceFirst(await recordTake())
  await expectReferenceFirst(await recordTake())
  await expect.poll(async () => (await savedTakes(page, texts[0])).length).toBe(3)
})

test('phone audio controls fit and the history dialog closes back to the same card', async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 320, height: 740 })
  const controls = card(page)
  await controls.getByRole('button', { name: 'Record a take' }).click()
  await controls.getByRole('button', { name: 'Stop recording' }).click()
  await expect.poll(async () => (await playedMedia(page)).length).toBe(1)
  await finishClip(page)
  await expect.poll(async () => (await playedMedia(page)).length).toBe(2)
  await finishClip(page)
  await expect.poll(async () => (await savedTakes(page, texts[0])).length).toBe(1)

  const actions = page.getByRole('button', { name: 'More actions' })
  const reference = controls.getByRole('button', { name: 'Ref' })
  const microphone = controls.getByRole('button', { name: 'Record a take' })
  const mine = controls.getByRole('button', { name: 'Mine' })
  for (const width of [320, 390]) {
    await page.setViewportSize({ width, height: 844 })
    const boxes = []
    for (const button of [reference, microphone, mine, actions]) {
      await expect(button).toBeVisible()
      const box = (await button.boundingBox())!
      expect(box.x).toBeGreaterThanOrEqual(0)
      expect(box.x + box.width).toBeLessThanOrEqual(width)
      expect(box.height).toBeGreaterThanOrEqual(44)
      boxes.push(box)
    }
    expect(boxes[1].y).toBeGreaterThanOrEqual(boxes[0].y + boxes[0].height)
    expect(boxes[1].width).toBeGreaterThan(boxes[0].width)
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
  await actions.click()
  await page.getByRole('button', { name: 'Recording history', exact: true }).click()
  const dialog = page.getByRole('dialog', { name: 'Recording history' })
  const recordings = dialog.getByRole('region', { name: 'Saved recordings' })
  await expect(recordings.locator('.take')).toHaveCount(1)
  await expect(controls.locator('.recording-history-panel')).toHaveCount(0)
  await page.keyboard.press('Escape')
  await expect(dialog).toHaveCount(0)
  await expect(actions).toBeFocused()
  await expect(microphone).toBeVisible()
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
  await card(page).getByRole('button', { name: 'Reveal', exact: true }).click()
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
  await expect(page.getByRole('region', { name: 'Library', exact: true })).toBeVisible()
  expect(await playedMedia(page)).toEqual([])
  await expect.poll(async () => (await savedTakes(page, texts[0])).length).toBe(1)
})

test('a graded card can record and replay a new take without changing the current card or grade', async ({
  page,
}) => {
  const controls = card(page)
  const phrase = page.getByTestId('practice-phrase')
  const previous = controls.getByRole('button', { name: 'Previous card', exact: true })
  const next = controls.getByRole('button', { name: 'Next card', exact: true })
  const microphone = controls.getByRole('button', { name: 'Record a take' })
  const good = controls.getByRole('button', { name: 'Good', exact: true })
  const current = async () => (await page.request.get('/api/practice/current')).json()

  await controls.getByRole('button', { name: 'Reveal', exact: true }).click()
  await expect(phrase).toHaveText(texts[0])
  const first = await current()
  await good.click()
  await expect(phrase).toHaveCount(0)
  await expect(previous).toBeEnabled()
  const nextCard = await current()
  await previous.click()
  await expect(phrase).toHaveText(texts[0])
  await expect(good).toHaveAttribute('aria-pressed', 'true')
  await expect(good).toBeDisabled()

  await microphone.click()
  await expect(next).toBeDisabled()
  await controls.getByRole('button', { name: 'Stop recording' }).click()
  await expect
    .poll(async () => (await playedMedia(page)).map((url) => new URL(url).pathname))
    .toEqual([first.card.answer.reference_url])
  await finishClip(page)
  await expect.poll(async () => (await playedMedia(page)).at(-1)).toMatch(/^blob:/)
  await finishClip(page)
  await expect.poll(async () => (await savedTakes(page, texts[0])).length).toBe(1)
  expect(await savedTakes(page, texts[1])).toEqual([])
  await expect(good).toBeDisabled()

  await next.click()
  await expect(phrase).toHaveCount(0)
  expect(await current()).toEqual(nextCard)
  await previous.click()
  await expect(phrase).toHaveText(texts[0])
  const beforeReplay = (await playedMedia(page)).length
  await controls.getByRole('button', { name: 'Mine', exact: true }).click()
  await expect.poll(async () => (await playedMedia(page)).length).toBe(beforeReplay + 1)
  await finishClip(page)
  const facts = await (await page.request.get('/api/practice/facts')).json()
  expect(facts.today).toMatchObject({ completions: 1, got_it: 1, recordings: 1 })
})
