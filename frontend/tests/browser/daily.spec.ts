import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { rateNew, revealAnswer, importPackage, playedMedia, savedTakes } from './helpers'

const cues = [
  { situation: 'Colleague asks: How is it going?' },
  { situation: 'Friend asks: How is the new job?', partner_line: 'So, how is it?' },
]
const phrase = (item_id: string, text: string, extra: object = {}) => ({
  item_id,
  text,
  explanation: `Meaning of ${text}`,
  detailed_explanation: `Longer help for ${text}`,
  cues: [{ situation: `Situation for ${text}` }],
  recall_hints: [`${item_id} hint`, `${item_id} clue`],
  usage_contexts: ['small talk'],
  ...extra,
})
const dailyPackage = {
  format: 'phrase-gym.package',
  version: 2,
  package_id: 'daily-browser-v1',
  source_id: 'daily-browser-source',
  source_label: 'Daily course',
  learning_language: 'en',
  card_language: 'en',
  help_language: 'en',
  lessons: [
    {
      lesson_id: 'small-talk',
      title: 'Small talk',
      phrases: [
        phrase('so-far', 'So far, so good.', { cues, recall_hints: ['far', 'so … good'] }),
        phrase('nice', 'Nice to meet you.'),
      ],
    },
  ],
}
const card = (page: Page) => page.locator('article.practice-card')
const modeLine = (page: Page) => page.getByTestId('mode-line')

async function rateBothThenReturnTomorrow(page: Page) {
  await page.goto('/')
  await importPackage(page, dailyPackage, 2)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  for (const text of ['So far, so good.', 'Nice to meet you.']) {
    await revealAnswer(page)
    await expect(card(page).getByRole('heading', { name: text })).toBeVisible()
    await rateNew(page)
  }
  await expect(page.getByText(/^Today's Daily practice is complete\. Next due: /)).toBeVisible()
  const tomorrow = new Date(Date.now() + 26 * 3600 * 1000).toISOString()
  await page.request.post(`/__test/clock/${tomorrow}`)
  await page.reload()
}

test.beforeEach(async ({ page }) => installFakeMedia(page))

test('silent Daily resumes a revealed card, rates and skips with automatic advance', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, dailyPackage, 2)
  await expect(card(page)).toHaveCount(0)
  await expect(page.getByTestId('lesson-counts')).toHaveText('0 due · 2 new')
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Daily · 1 / 5 new')
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await expect(card(page).locator('.practice-prompt')).toBeVisible()
  await expect(card(page).getByRole('button', { name: 'Next' })).toHaveCount(0)
  await expect(card(page).getByRole('button', { name: 'Again' })).toHaveCount(0)
  await rateNew(page)
  await expect(modeLine(page)).toHaveText('Daily · 2 / 5 new')
  await rateNew(page)
  await expect(page.getByTestId('lesson-counts')).toHaveText(/^Nothing waiting today · Next due: /)

  const tomorrow = new Date(Date.now() + 26 * 3600 * 1000).toISOString()
  await page.request.post(`/__test/clock/${tomorrow}`)
  await page.reload()
  await expect(page.getByTestId('lesson-counts')).toHaveText('2 due · 0 new')
  await expect(card(page)).toHaveCount(0)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Daily · 1 / 20 due')
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  const prompt = card(page).locator('.practice-prompt')
  const firstCue = await prompt.textContent()
  await card(page).getByRole('button', { name: 'New cue' }).click()
  await expect(prompt).not.toHaveText(firstCue ?? '')
  await card(page).getByRole('button', { name: 'Hint' }).click()
  await expect(card(page).getByText(/^Hint: (far|so … good)$/)).toBeVisible()
  await expect(card(page)).not.toContainText('So far, so good.')
  await card(page).getByRole('button', { name: 'Show' }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText('So far, so good.')

  await page.reload()
  await expect(card(page)).toHaveCount(0)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText('So far, so good.')
  await page.getByRole('button', { name: 'Good' }).click()
  await expect(modeLine(page)).toHaveText('Daily · 2 / 20 due')
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await card(page).getByRole('button', { name: 'Show', exact: true }).click()
  await card(page).getByRole('button', { name: 'More explanation ▾' }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText('Nice to meet you.')
  await expect(card(page).getByText('Longer help for Nice to meet you.')).toBeVisible()
  await page.getByRole('button', { name: 'Skip' }).click()
  await expect(page.getByText("Today's Daily practice is complete.", { exact: true })).toBeVisible()
  await expect(page.getByTestId('lesson-counts')).toHaveText('1 due · 0 new')
})

test('recorded Daily keeps front and rehearsal takes and saves late uploads to their phrase', async ({
  page,
}) => {
  await rateBothThenReturnTomorrow(page)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await card(page).getByRole('button', { name: 'Record a take' }).click()
  await card(page).getByRole('button', { name: 'Stop recording' }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText('So far, so good.')
  await expect.poll(() => playedMedia(page)).toEqual([expect.stringMatching(/\/reference$/)])
  await page.evaluate(() =>
    (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia[0]!.dispatchEvent(
      new Event('ended'),
    ),
  )
  await expect
    .poll(() => playedMedia(page))
    .toEqual([expect.stringMatching(/\/reference$/), expect.stringMatching(/^blob:/)])
  await page.evaluate(() =>
    (window as unknown as { __playedMedia: HTMLMediaElement[] }).__playedMedia
      .at(-1)!
      .dispatchEvent(new Event('ended')),
  )
  await expect(card(page).getByRole('button', { name: 'Mine' })).toBeVisible()
  await expect(card(page).getByRole('button', { name: 'Compare' })).toBeVisible()

  let releaseUpload!: () => void
  const uploadHeld = new Promise<void>((resolve) => (releaseUpload = resolve))
  await page.route('**/api/recordings', async (route) => {
    if (route.request().method() === 'POST') await uploadHeld
    await route.continue()
  })
  await card(page).getByRole('button', { name: 'Record a take' }).click()
  await expect(card(page).getByRole('button', { name: 'Stop recording' })).toBeVisible()
  await page.getByRole('button', { name: 'Hard' }).click()
  await expect(modeLine(page)).toHaveText('Daily · 2 / 20 due')
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await expect(card(page).getByRole('button', { name: 'Record a take' })).toBeVisible()
  await expect(page.getByText(/take\(s\) waiting to save/)).toBeVisible()
  releaseUpload()
  await expect(page.getByText(/take\(s\) waiting to save/)).toHaveCount(0)
  const takes = await savedTakes(page, 'So far, so good.')
  expect(takes).toHaveLength(2)
  expect(takes.map((take: { mode: string; kind: string }) => [take.mode, take.kind])).toEqual([
    ['daily', 'recall'],
    ['daily', 'recall'],
  ])
  expect(takes.every((take: { answer: string }) => take.answer === 'So far, so good.')).toBe(true)
  await expect(modeLine(page)).toHaveText('Daily · 2 / 20 due')
})

test('Delete on a revealed back confirms, then advances without a grade', async ({ page }) => {
  await rateBothThenReturnTomorrow(page)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await card(page).getByRole('button', { name: 'Show' }).click()
  await card(page).getByRole('button', { name: 'Delete' }).click()
  const confirm = page.getByRole('dialog', { name: 'Delete “So far, so good.”?' })
  await confirm.getByRole('button', { name: 'Cancel' }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText('So far, so good.')
  await card(page).getByRole('button', { name: 'Delete' }).click()
  await confirm.getByRole('button', { name: 'Delete permanently' }).click()
  await expect(modeLine(page)).toHaveText('Daily · 1 / 20 due')
  await expect(card(page).locator('.practice-prompt')).toHaveText('Situation for Nice to meet you.')
})

test.describe('mobile', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true })

  test('mobile Daily controls fit and respond to taps', async ({ page }) => {
    await page.goto('/')
    await importPackage(page, dailyPackage, 2, true)
    await page.getByRole('button', { name: 'Daily', exact: true }).tap()
    await revealAnswer(page, true)
    await card(page).getByRole('button', { name: 'Record a take' }).tap()
    await card(page).getByRole('button', { name: 'Stop recording' }).tap()
    await expect(card(page).getByRole('button', { name: 'Mine' })).toBeVisible()
    const before = (await playedMedia(page)).length
    await card(page).getByRole('button', { name: 'Compare' }).tap()
    await expect
      .poll(async () => (await playedMedia(page)).slice(before))
      .toEqual([expect.stringMatching(/\/reference$/)])
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
    await rateNew(page, true)
    await revealAnswer(page)
    await expect(card(page).getByRole('heading', { name: 'Nice to meet you.' })).toBeVisible()
    const tomorrow = new Date(Date.now() + 26 * 3600 * 1000).toISOString()
    await rateNew(page, true)
    await page.request.post(`/__test/clock/${tomorrow}`)
    await page.getByRole('button', { name: 'Daily', exact: true }).tap()
    await card(page).getByRole('button', { name: 'Show' }).tap()
    for (const name of ['Again', 'Hard', 'Good', 'Easy', 'Skip', 'Record a take'])
      await expect(card(page).getByRole('button', { name })).toBeInViewport({ ratio: 0.5 })
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
    await card(page).getByRole('button', { name: 'Again' }).tap()
    await expect(modeLine(page)).toHaveText('Daily · 2 / 20 due')
  })
})

test('Easy uses the fourth FSRS rating and persists in practice facts', async ({ page }) => {
  await rateBothThenReturnTomorrow(page)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(card(page).getByRole('button', { name: 'Easy', exact: true })).toHaveCount(0)
  await card(page).getByRole('button', { name: 'Show', exact: true }).click()
  for (const name of ['Again', 'Hard', 'Good', 'Easy'])
    await expect(card(page).getByRole('button', { name, exact: true })).toBeVisible()
  await card(page).getByRole('button', { name: 'Easy', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Daily · 2 / 20 due')
  const response = await page.request.get('/api/practice/facts')
  const facts = await response.json()
  expect(facts.today.easy).toBe(1)
  expect(facts.work.easy).toBe(1)
  await page.reload()
  await expect(page.getByTestId('today-ratings')).toHaveText('1 Easy')
})

test('new-card Skip consumes a new slot and leaves the phrase unrated for a later pass', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, dailyPackage, 2)
  await page.getByRole('button', { name: 'Daily', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Daily · 1 / 5 new')
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await card(page).getByRole('button', { name: 'Show', exact: true }).click()
  await card(page).getByRole('button', { name: 'More explanation ▾' }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText('So far, so good.')
  await card(page).getByRole('button', { name: 'Skip', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Daily · 2 / 5 new')
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await rateNew(page)
  await expect(card(page)).toHaveCount(0)
  const response = await page.request.get('/api/practice/facts')
  const facts = await response.json()
  expect(facts.today).toMatchObject({ new_attempts: 1, recalls: 0, skipped: 1, not_yet: 1 })
  expect(facts.library).toMatchObject({ new_count: 1, later_count: 1, due_count: 0 })
  await page.getByRole('button', { name: 'Extra', exact: true }).click()
  await expect(modeLine(page)).toHaveText('Extra · 3 / 5 new')
  await card(page).getByRole('button', { name: 'Show' }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText('So far, so good.')
})
