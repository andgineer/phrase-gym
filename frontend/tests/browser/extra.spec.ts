import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { importPackage, playedMedia, savedTakes } from './helpers'

const phrase = (item_id: string, text: string) => ({
  item_id,
  text,
  explanation: `Meaning of ${text}`,
  detailed_explanation: `Longer help for ${text}`,
  cues: [{ situation: `Situation for ${text}` }],
  recall_hints: [`${item_id} hint`, `${item_id} clue`],
  usage_contexts: ['small talk'],
})
const texts = ['So far, so good.', 'Nice to meet you.', 'That works for me.']
const extraPackage = {
  format: 'phrase-gym.package',
  version: 2,
  package_id: 'extra-browser-v1',
  source_id: 'extra-browser-source',
  source_label: 'Extra course',
  learning_language: 'en',
  card_language: 'en',
  help_language: 'en',
  lessons: [
    {
      lesson_id: 'small-talk',
      title: 'Small talk',
      phrases: texts.map((text, index) => phrase(`item-${index}`, text)),
    },
  ],
}
const card = (page: Page) => page.locator('article.practice-card')
const modeLine = (page: Page) => page.getByTestId('mode-line')
const stopButton = (page: Page) => page.getByRole('button', { name: 'Stop', exact: true })
const details = (page: Page) => page.locator('section.phrase-details')

async function setLimits(page: Page, due: number, fresh: number) {
  const settings = await (await page.request.get('/api/settings')).json()
  const response = await page.request.put('/api/settings', {
    data: { ...settings, daily_due_limit: due, daily_new_limit: fresh },
  })
  expect(response.ok()).toBe(true)
}

async function openDetails(page: Page, text: string, tap = false) {
  const button = page.getByRole('button', { name: `Open details: ${text}` })
  await (tap ? button.tap() : button.click())
  await expect(details(page).getByRole('heading', { name: text })).toBeVisible()
}

test.beforeEach(async ({ page }) => installFakeMedia(page))

test('Extra extends a finished Daily, Stop keeps the take and a repeat press does not compound', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 'Added 3 phrases.')
  await setLimits(page, 1, 1)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Daily · 1 / 1 new')
  await expect(stopButton(page)).toHaveCount(0)
  await page.getByRole('button', { name: 'Next' }).click()
  await expect(page.getByText("Today's Daily practice is complete.", { exact: true })).toBeVisible()

  await page.getByRole('button', { name: 'Extra', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Extra · 2 / 2 new')
  await expect(card(page).getByRole('heading', { name: texts[1] })).toBeVisible()
  let releaseUpload!: () => void
  const uploadHeld = new Promise<void>((resolve) => (releaseUpload = resolve))
  await page.route('**/api/recordings', async (route) => {
    if (route.request().method() === 'POST') await uploadHeld
    await route.continue()
  })
  await card(page).getByRole('button', { name: 'Record a take' }).click()
  await expect(card(page).getByRole('button', { name: 'Stop recording' })).toBeVisible()
  await stopButton(page).click()
  await expect(card(page)).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Extra', exact: true })).toBeVisible()
  releaseUpload()
  await expect.poll(async () => (await savedTakes(page, texts[1])).length).toBe(1)
  const [take] = await savedTakes(page, texts[1])
  expect([take.mode, take.kind, take.answer]).toEqual(['extra', 'first_study', texts[1]])

  await page.getByRole('button', { name: 'Extra', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Extra · 2 / 2 new')
  await expect(card(page).getByRole('heading', { name: texts[1] })).toBeVisible()
  await page.getByRole('button', { name: 'Next' }).click()
  await expect(page.getByText("Today's Extra practice is complete.", { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Extra', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Extra · 3 / 3 new')
  await page.getByRole('button', { name: 'Next' }).click()
  await page.getByRole('button', { name: 'Extra', exact: true }).click()
  await expect(page.getByText(/^Nothing waiting today\. Next due: /)).toBeVisible()
  await expect(card(page)).toHaveCount(0)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(card(page)).toHaveCount(0)
})

test('Practice from details rates or studies one phrase and returns to its details', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, extraPackage, 'Added 3 phrases.')
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(card(page).getByRole('heading', { name: texts[0] })).toBeVisible()
  await page.getByRole('button', { name: 'Next' }).click()
  await expect(card(page).getByRole('heading', { name: texts[1] })).toBeVisible()
  await page.getByRole('button', { name: 'Practice', exact: true }).click()

  await openDetails(page, texts[0])
  const tomorrow = await details(page)
    .getByText(/^Next due: /)
    .textContent()
  await details(page).getByRole('button', { name: 'Practice' }).click()
  await expect(modeLine(page)).toHaveText('Practice')
  await expect(stopButton(page)).toHaveCount(0)
  await expect(card(page).locator('.practice-prompt')).toHaveText(`Situation for ${texts[0]}`)
  await card(page).getByRole('button', { name: 'Show' }).click()
  await page.getByRole('button', { name: 'Got it' }).click()
  await expect(details(page).getByRole('heading', { name: texts[0] })).toBeVisible()
  await expect(details(page).getByText(/^Next due: /)).not.toHaveText(tomorrow ?? '')

  await details(page).getByRole('button', { name: '← Back' }).click()
  await openDetails(page, texts[1])
  await expect(details(page).getByText('New: not studied yet')).toBeVisible()
  await details(page).getByRole('button', { name: 'Practice' }).click()
  await expect(card(page).getByText('First meeting · no rating')).toBeVisible()
  await page.getByRole('button', { name: 'Next' }).click()
  await expect(details(page).getByText(tomorrow ?? '')).toBeVisible()
  await details(page).getByRole('button', { name: '← Back' }).click()

  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Daily · 2 / 5 new')
  await expect(card(page).getByRole('heading', { name: texts[2] })).toBeVisible()

  await page.getByRole('button', { name: 'Practice', exact: true }).click()
  await openDetails(page, texts[1])
  await details(page).getByRole('button', { name: 'Practice' }).click()
  await card(page).getByRole('button', { name: 'Show' }).click()
  await card(page).getByRole('button', { name: 'Delete' }).click()
  await page
    .getByRole('dialog', { name: `Delete “${texts[1]}”?` })
    .getByRole('button', { name: 'Delete permanently' })
    .click()
  await expect(card(page)).toHaveCount(0)
  await expect(details(page)).toHaveCount(0)
  await expect(page.getByRole('button', { name: `Open details: ${texts[1]}` })).toHaveCount(0)
  await expect(page.getByRole('button', { name: `Open details: ${texts[0]}` })).toBeVisible()
})

test.describe('mobile', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true })

  test('mobile Extra and Practice capture and replay takes', async ({ page }) => {
    await page.goto('/')
    await importPackage(page, extraPackage, 'Added 3 phrases.', true)
    await page.getByRole('button', { name: 'Extra', exact: true }).tap()
    await expect(modeLine(page)).toHaveText('Extra · 1 / 5 new')
    const stop = stopButton(page)
    await expect(stop).toBeInViewport()
    await card(page).getByRole('button', { name: 'Record a take' }).tap()
    await card(page).getByRole('button', { name: 'Stop recording' }).tap()
    await expect(card(page).getByRole('button', { name: '▶ Mine' })).toBeVisible()
    const before = (await playedMedia(page)).length
    await card(page).getByRole('button', { name: 'Compare' }).tap()
    await expect
      .poll(async () => (await playedMedia(page)).slice(before))
      .toEqual([expect.stringMatching(/^blob:/)])
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
    await stop.tap()
    await expect(card(page)).toHaveCount(0)

    await openDetails(page, texts[0], true)
    await details(page).getByRole('button', { name: 'Practice' }).tap()
    await expect(modeLine(page)).toHaveText('Practice')
    await card(page).getByRole('button', { name: 'Record a take' }).tap()
    await card(page).getByRole('button', { name: 'Stop recording' }).tap()
    await expect(card(page).getByRole('button', { name: '▶ Mine' })).toBeVisible()
    await page.getByRole('button', { name: 'Next' }).tap()
    await expect(details(page).getByRole('heading', { name: texts[0] })).toBeVisible()
    await expect
      .poll(async () =>
        (await savedTakes(page, texts[0])).map((take: { mode: string }) => take.mode),
      )
      .toEqual(['practice', 'extra'])
    await details(page).getByRole('button', { name: '← Back' }).tap()
    await openDetails(page, texts[0], true)
    const history = details(page).locator('.take-history li')
    await expect(history).toHaveCount(2)
    await expect(history.first()).toContainText('Practice · First meeting')
    const replayed = (await playedMedia(page)).length
    await history.first().getByRole('button', { name: '▶ Mine' }).tap()
    await expect
      .poll(async () => (await playedMedia(page)).slice(replayed))
      .toEqual([expect.stringMatching(/\/api\/recordings\/.+\/audio$/)])
  })
})
