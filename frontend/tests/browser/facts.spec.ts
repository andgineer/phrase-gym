import type { Locator, Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { card, details, modeLine, openDetails } from './extra-package'
import { importPackage, playedMedia } from './helpers'

const texts = [
  'So far, so good.',
  'Nice to meet you.',
  'That works for me.',
  'Let me check.',
  'Sounds good.',
  'See you soon.',
]
const factsPackage = {
  format: 'phrase-gym.package',
  version: 2,
  package_id: 'facts-browser-v1',
  source_id: 'facts-browser-source',
  source_label: 'Facts course',
  learning_language: 'en',
  card_language: 'en',
  help_language: 'en',
  lessons: [
    {
      lesson_id: 'small-talk',
      title: 'Small talk',
      phrases: texts.map((text, index) => ({
        item_id: `item-${index}`,
        text,
        explanation: `Meaning of ${text}`,
        cues: [{ situation: `Situation for ${text}` }],
        recall_hints: [`item-${index} hint`, `item-${index} clue`],
        usage_contexts: ['small talk'],
      })),
    },
  ],
}
const pagedLessons = Array.from({ length: 21 }, (_, index) => ({
  lesson_id: `unit-${index + 1}`,
  title: `Unit ${index + 1}`,
  phrases: [
    {
      ...factsPackage.lessons[0]!.phrases[0]!,
      item_id: `paged-${index + 1}`,
      text: `Paged phrase ${index + 1}.`,
      cues: [{ situation: `Paged situation ${index + 1}` }],
    },
  ],
}))
// One package holds at most 20 lessons, so the 21st lesson arrives separately.
const pagedPackages = [pagedLessons.slice(0, 20), pagedLessons.slice(20)].map((lessons, part) => ({
  ...factsPackage,
  package_id: `facts-paged-${part}-v1`,
  source_id: 'facts-paged-source',
  lessons,
}))
const latePackage = {
  ...factsPackage,
  package_id: 'facts-late-v1',
  source_id: 'facts-late-source',
  lessons: [{ ...factsPackage.lessons[0]!, lesson_id: 'late', title: 'Late lesson' }],
}
latePackage.lessons[0]!.phrases = [{ ...factsPackage.lessons[0]!.phrases[0]!, text: 'Late one.' }]
const facts = (page: Page) => page.getByRole('region', { name: 'Practice facts' })
const fact = (page: Page, id: string) => page.getByTestId(id)
const press = (page: Page, name: string) => page.getByRole('button', { name, exact: true }).click()
const todayHeading = (page: Page) =>
  facts(page).getByRole('heading', { name: 'Today’s practice ✦' })

async function setLimits(page: Page, due: number, fresh: number) {
  const settings = await (await page.request.get('/api/settings')).json()
  const response = await page.request.put('/api/settings', {
    data: { ...settings, daily_due_limit: due, daily_new_limit: fresh },
  })
  expect(response.ok()).toBe(true)
}

async function meet(page: Page, text: string) {
  await expect(card(page).getByRole('heading', { name: text })).toBeVisible()
  await press(page, 'Next')
}

async function expectStudied(item: Locator, text: string, outcome: string) {
  await expect(item.locator('strong')).toHaveText(text)
  await expect(item.locator('.muted')).toHaveText(outcome)
}

// Monday 10:00 in Europe/Belgrade, far from any learning-day boundary.
const start = Date.parse('2026-11-02T09:00:00Z')

async function setClock(page: Page, hours: number) {
  const instant = new Date(start + hours * 3600 * 1000)
  await page.request.post(`/__test/clock/${instant.toISOString()}`)
  // Takes are stamped by the browser, so its clock follows the server's.
  await page.clock.setFixedTime(instant)
}

async function returnToPage(page: Page) {
  await page.evaluate(() => {
    document.dispatchEvent(new Event('visibilitychange'))
    window.dispatchEvent(new Event('focus'))
  })
}

async function moveClock(page: Page, hours: number) {
  await setClock(page, hours)
  await page.reload()
}

test.beforeEach(async ({ page }) => {
  await installFakeMedia(page)
  await setClock(page, 0)
})

test('completion returns to Today facts that continue through limits, Extra and uploads', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, factsPackage, 'Added 6 phrases.')
  await expect(facts(page)).toHaveCount(0)
  await setLimits(page, 1, 1)
  await press(page, 'Daily')
  await meet(page, texts[0])
  await expect(page.getByText("Today's Daily practice is complete.", { exact: true })).toBeVisible()
  await expect(todayHeading(page)).toBeVisible()
  await expect(fact(page, 'today-counts')).toHaveText('1 completion · 1 first meeting')
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')
  await expect(fact(page, 'best-streak')).toHaveText('Personal best: 1 day')
  await expect(fact(page, 'recent-days').getByRole('listitem')).toHaveCount(7)
  await expect(fact(page, 'recent-days').getByRole('listitem').last()).toHaveAccessibleName(
    /: practiced$/,
  )
  await expect(fact(page, 'your-work')).toHaveText(
    '1 completion · 1 first meeting · 1 day with practice',
  )
  await expect(fact(page, 'next-return')).toHaveText('Next return: Tomorrow · 1 phrase due')
  const panel = await facts(page).boundingBox()
  const daily = await page.getByRole('button', { name: 'Daily', exact: true }).boundingBox()
  expect(daily!.y).toBeGreaterThan(panel!.y + panel!.height)

  await page.getByLabel('Search lessons').fill('Small')
  await expect(page.getByLabel('Lesson to drill')).toContainText('Small talk · Facts course')
  await page.getByLabel('Find a phrase').fill(texts[2])
  await expect(page.getByRole('button', { name: `Open details: ${texts[2]}` })).toBeVisible()
  await page.getByLabel('Find a phrase').fill('')

  await setLimits(page, 1, 2)
  await press(page, 'Daily')
  await expect(modeLine(page)).toHaveText('Daily · 2 / 2 new')
  await meet(page, texts[1])
  await expect(fact(page, 'today-counts')).toHaveText('2 completions · 2 first meetings')

  let releaseUpload!: () => void
  const uploadHeld = new Promise<void>((resolve) => (releaseUpload = resolve))
  await page.route('**/api/recordings', async (route) => {
    if (route.request().method() === 'POST') await uploadHeld
    await route.continue()
  })
  await press(page, 'Extra')
  await expect(modeLine(page)).toHaveText('Extra · 3 / 4 new')
  await card(page).getByRole('button', { name: 'Record a take' }).click()
  await expect(card(page).getByRole('button', { name: 'Stop recording' })).toBeVisible()
  await meet(page, texts[2])
  await meet(page, texts[3])
  await expect(page.getByText("Today's Extra practice is complete.", { exact: true })).toBeVisible()
  await expect(fact(page, 'pending-uploads')).toHaveText(
    '1 recording waiting to upload from this device',
  )
  await expect(fact(page, 'today-recordings')).toHaveCount(0)
  releaseUpload()
  await expect(fact(page, 'pending-uploads')).toHaveCount(0)
  await expect(fact(page, 'today-recordings')).toHaveText('1 recording available')

  await press(page, 'Extra')
  await expect(modeLine(page)).toHaveText('Extra · 5 / 6 new')
  await meet(page, texts[4])
  await meet(page, texts[5])
  await expect(fact(page, 'today-counts')).toHaveText('6 completions · 6 first meetings')
  await expect(fact(page, 'next-return')).toHaveText('Next return: Tomorrow · 6 phrases due')

  await press(page, 'See today’s phrases')
  const studied = facts(page).getByRole('list', { name: 'Today’s phrases' }).getByRole('listitem')
  await expect(studied).toHaveCount(5)
  await expectStudied(studied.first(), texts[0], 'first meeting · Daily · Small talk')
  await expectStudied(studied.nth(2), texts[2], 'first meeting · Extra · Small talk')
  await facts(page).getByRole('button', { name: 'More', exact: true }).click()
  await expect(studied).toHaveCount(6)
  await expect(facts(page).getByRole('button', { name: 'More', exact: true })).toHaveCount(0)

  await page.getByRole('button', { name: `Delete: ${texts[0]}` }).click()
  await page
    .getByRole('dialog', { name: `Delete “${texts[0]}”?` })
    .getByRole('button', { name: 'Delete permanently' })
    .click()
  await expectStudied(studied.first(), texts[0], 'first meeting · Daily · Small talk · deleted')
  await expect(fact(page, 'today-counts')).toHaveText('6 completions · 6 first meetings')
  await expect(fact(page, 'next-return')).toHaveText('Next return: Tomorrow · 5 phrases due')

  await openDetails(page, texts[2])
  const history = details(page).locator('.take-history li')
  await expect(history).toHaveCount(1)
  await expect(history).toContainText('Extra · First meeting')
  const before = (await playedMedia(page)).length
  await history.getByRole('button', { name: '▶ Mine' }).click()
  await expect
    .poll(async () => (await playedMedia(page)).slice(before))
    .toEqual([expect.stringMatching(/\/api\/recordings\/.+\/audio$/)])
})

test('next-day entry keeps yesterday’s streak until practice and resets after a missed day', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, factsPackage, 'Added 6 phrases.')
  await setLimits(page, 1, 1)
  await press(page, 'Daily')
  await meet(page, texts[0])
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')

  await moveClock(page, 24)
  await expect(todayHeading(page)).toHaveCount(0)
  await expect(fact(page, 'today-counts')).toHaveCount(0)
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')
  const days = fact(page, 'recent-days').getByRole('listitem')
  await expect(days.last()).toHaveAccessibleName(/: no practice$/)
  await expect(days.nth(5)).toHaveAccessibleName(/: practiced$/)
  await expect(fact(page, 'next-return')).toHaveText('Next return: Today · 1 phrase due')
  await expect(page.getByTestId('lesson-counts')).toHaveText('1 due · 5 new')

  await press(page, 'Daily')
  await expect(modeLine(page)).toHaveText('Daily · 1 / 1 due')
  await card(page).getByRole('button', { name: 'Show' }).click()
  await press(page, 'Got it')
  await meet(page, texts[1])
  await expect(todayHeading(page)).toBeVisible()
  await expect(fact(page, 'streak')).toHaveText('🔥 2-day practice streak')
  await expect(fact(page, 'today-counts')).toHaveText('2 completions · 1 first meeting')
  await expect(fact(page, 'today-ratings')).toHaveText('1 Got it')

  await press(page, 'Extra')
  await meet(page, texts[2])
  await expect(page.getByText("Today's Extra practice is complete.", { exact: true })).toBeVisible()
  await expect(fact(page, 'today-counts')).toHaveText('3 completions · 2 first meetings')
  await page.getByLabel('Find a phrase').fill(texts[3])
  await expect(page.getByRole('button', { name: `Open details: ${texts[3]}` })).toBeVisible()

  await moveClock(page, 72)
  await expect(todayHeading(page)).toHaveCount(0)
  await expect(fact(page, 'streak')).toHaveText('No current practice streak')
  await expect(fact(page, 'best-streak')).toHaveText('Personal best: 2 days')
  await expect(fact(page, 'your-work')).toHaveText(
    '4 completions · 3 first meetings · 2 days with practice',
  )
  await expect(fact(page, 'next-return')).toHaveText('Next return: Today · 3 phrases due')
  await press(page, 'Daily')
  await expect(modeLine(page)).toHaveText('Daily · 1 / 1 due')
})

test('an open entry recomputes facts and counts when it returns on a new learning day', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, factsPackage, 'Added 6 phrases.')
  await setLimits(page, 1, 1)
  await press(page, 'Daily')
  await meet(page, texts[0])
  const message = page.getByText("Today's Daily practice is complete.", { exact: true })
  await expect(message).toBeVisible()
  await expect(fact(page, 'today-counts')).toHaveText('1 completion · 1 first meeting')
  await expect(fact(page, 'next-return')).toHaveText('Next return: Tomorrow · 1 phrase due')
  const counts = page.getByTestId('lesson-counts')
  await expect(counts).toHaveText('0 due · 5 new')
  await press(page, 'See today’s phrases')
  const studied = facts(page).getByRole('list', { name: 'Today’s phrases' }).getByRole('listitem')
  await expect(studied).toHaveCount(1)

  await setClock(page, 24)
  const listed = page.waitForRequest((request) =>
    request.url().includes('/api/practice/studied?learning_date=2026-11-03'),
  )
  await page.evaluate(() => {
    document.dispatchEvent(new Event('visibilitychange'))
    window.dispatchEvent(new Event('focus'))
  })
  await listed
  await expect(todayHeading(page)).toHaveCount(0)
  await expect(fact(page, 'today-counts')).toHaveCount(0)
  await expect(message).toHaveCount(0)
  await expect(studied).toHaveCount(0)
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')
  const days = fact(page, 'recent-days').getByRole('listitem')
  await expect(days.last()).toHaveAccessibleName('Tue: no practice')
  await expect(days.nth(5)).toHaveAccessibleName('Mon: practiced')
  await expect(fact(page, 'next-return')).toHaveText('Next return: Today · 1 phrase due')
  await expect(counts).toHaveText('1 due · 5 new')
  await importPackage(page, latePackage, 'Added 1 phrases.')
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')
  await expect(message).toHaveCount(0)

  await press(page, 'Daily')
  await card(page).getByRole('button', { name: 'Show' }).click()
  await press(page, 'Got it')
  await meet(page, texts[1])
  await expect(fact(page, 'today-counts')).toHaveText('2 completions · 1 first meeting')
  await press(page, 'See today’s phrases')
  await expect(studied).toHaveCount(2)
  await expectStudied(studied.first(), texts[0], 'Got it · Daily · Small talk')
  await expectStudied(studied.last(), texts[1], 'first meeting · Daily · Small talk')
})

test('returning on the same day keeps loaded pages and a later lesson; a new day resets them', async ({
  page,
}) => {
  const requests: string[] = []
  page.on('request', (request) =>
    requests.push(new URL(request.url()).pathname + new URL(request.url()).search),
  )
  const count = (prefix: string) => requests.filter((path) => path.startsWith(prefix)).length
  await page.goto('/')
  await importPackage(page, pagedPackages[0]!, 'Added 20 phrases.')
  await importPackage(page, pagedPackages[1]!, 'Added 1 phrases.')
  await setLimits(page, 1, 6)
  await press(page, 'Daily')
  for (let step = 1; step <= 6; step++) {
    await expect(modeLine(page)).toHaveText(`Daily · ${step} / 6 new`)
    await press(page, 'Next')
  }
  await expect(fact(page, 'today-counts')).toHaveText('6 completions · 6 first meetings')
  await press(page, 'See today’s phrases')
  const studied = facts(page).getByRole('list', { name: 'Today’s phrases' }).getByRole('listitem')
  await expect(studied).toHaveCount(5)
  await facts(page).getByRole('button', { name: 'More', exact: true }).click()
  await expect(studied).toHaveCount(6)
  await press(page, 'More lessons')
  const choice = page.getByLabel('Lesson to drill')
  await expect(choice.locator('option')).toHaveCount(22)
  await choice.selectOption({ label: 'Unit 21 · Facts course' })
  const counts = page.getByTestId('lesson-counts')
  await expect(counts).toHaveText('0 due · 1 new')

  const loads = count('/api/practice/facts')
  const lists = count('/api/practice/studied')
  const lessons = count('/api/lessons')
  await returnToPage(page)
  await expect.poll(() => count('/api/practice/facts')).toBe(loads + 2)
  await expect(page.getByRole('button', { name: 'More lessons' })).toHaveCount(0)
  await expect(choice.locator('option')).toHaveCount(22)
  await expect(choice).toHaveValue(/.+/)
  await expect(counts).toHaveText('0 due · 1 new')
  await expect(studied).toHaveCount(6)
  expect([count('/api/practice/studied'), count('/api/lessons')]).toEqual([lists, lessons])

  await setClock(page, 24)
  const newList = page.waitForRequest((request) =>
    request.url().includes('/api/practice/studied?learning_date=2026-11-03'),
  )
  const refreshed = page.waitForRequest((request) =>
    request.url().includes('/api/lessons?query=&offset=0&limit=21'),
  )
  await returnToPage(page)
  await Promise.all([newList, refreshed])
  await expect(todayHeading(page)).toHaveCount(0)
  await expect(studied).toHaveCount(0)
  await expect(fact(page, 'next-return')).toHaveText('Next return: Today · 6 phrases due')
  await expect(choice.locator('option')).toHaveCount(22)
  await expect(counts).toHaveText('0 due · 1 new')
})

test.describe('mobile', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true })

  test('mobile Today facts fit above the entry controls', async ({ page }) => {
    await page.goto('/')
    await importPackage(page, factsPackage, 'Added 6 phrases.', true)
    await setLimits(page, 1, 1)
    await page.getByRole('button', { name: 'Daily', exact: true }).tap()
    await expect(card(page).getByRole('heading', { name: texts[0] })).toBeVisible()
    await page.getByRole('button', { name: 'Next', exact: true }).tap()
    await expect(fact(page, 'today-counts')).toHaveText('1 completion · 1 first meeting')
    await facts(page).getByRole('button', { name: 'See today’s phrases' }).tap()
    await expect(facts(page).getByRole('listitem').last()).toContainText(texts[0])
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
    await page.getByRole('button', { name: 'Extra', exact: true }).tap()
    await expect(modeLine(page)).toHaveText('Extra · 2 / 2 new')
  })
})
