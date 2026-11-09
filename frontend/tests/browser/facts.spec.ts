import type { Locator, Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { card, details, modeLine, openDetails } from './extra-package'
import {
  rateNew,
  revealAnswer,
  importPackage,
  libraryPhrase,
  phraseSearch,
  playedMedia,
} from './helpers'

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
async function press(page: Page, name: string) {
  await page.getByRole('button', { name, exact: true }).click()
  if (['Again', 'Hard', 'Good', 'Easy', 'Skip'].includes(name))
    await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
}

async function setLimits(page: Page, due: number, fresh: number) {
  const settings = await (await page.request.get('/api/settings')).json()
  const response = await page.request.put('/api/settings', {
    data: { ...settings, daily_due_limit: due, daily_new_limit: fresh },
  })
  expect(response.ok()).toBe(true)
}

async function meet(page: Page, text: string) {
  await revealAnswer(page)
  await expect(card(page).getByRole('heading', { name: text })).toBeVisible()
  await rateNew(page)
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
  await importPackage(page, factsPackage, 6)
  await expect(facts(page)).toHaveCount(0)
  await setLimits(page, 1, 1)
  await press(page, 'Daily')
  await meet(page, texts[0])
  await expect(page.getByText("Today's Daily practice is complete.", { exact: true })).toBeVisible()
  await expect(facts(page).getByRole('heading', { name: 'Today’s practice ✦' })).toHaveCount(0)
  await expect(fact(page, 'today-title')).toHaveText('Today — 1 new')
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')
  await expect(fact(page, 'best-streak')).toHaveText('Personal best: 1 day')
  await expect(page.getByRole('list', { name: 'Last seven days' })).toHaveCount(0)
  await expect(fact(page, 'progress-title')).toHaveText('Progress — 1 day')
  await expect(fact(page, 'progress')).toHaveText('1 new phrase')
  await expect(fact(page, 'progress-ratings')).toHaveText('1 Again')
  await expect(fact(page, 'total-title')).toHaveText('Total — 6 phrases')
  await expect(fact(page, 'library')).toHaveText('1 later · 5 new')
  await expect(fact(page, 'long-term')).toHaveCount(0)
  const panel = await facts(page).boundingBox()
  const daily = await page.getByRole('button', { name: 'Daily', exact: true }).boundingBox()
  expect(daily!.y).toBeGreaterThan(panel!.y + panel!.height)

  const lesson = page.getByRole('combobox', { name: 'Lesson', exact: true })
  await lesson.click()
  await lesson.pressSequentially('Small')
  await expect(page.getByRole('option')).toHaveText(['All lessons', 'Small talk · Facts course'])
  await expect(lesson).toHaveAttribute('aria-activedescendant', /.+/)
  await lesson.press('Escape')
  await expect(page.getByRole('option')).toHaveCount(0)
  await expect(lesson).toHaveValue('')
  await lesson.pressSequentially('Sm')
  await expect(page.getByRole('option')).toHaveText(['All lessons', 'Small talk · Facts course'])
  await expect(lesson).toHaveAttribute('aria-activedescendant', /.+/)
  await lesson.press('Enter')
  await expect(lesson).toHaveValue('Small talk · Facts course')
  await expect(lesson).toBeFocused()
  await lesson.press('ArrowDown')
  await lesson.press('ArrowUp')
  await lesson.press('Enter')
  await expect(lesson).toHaveValue('')
  await expect(page.getByTestId('lesson-counts')).toHaveText('0 due · 5 new')
  await page.getByRole('navigation').getByRole('button', { name: 'Library', exact: true }).click()
  await phraseSearch(page).fill(texts[2])
  await expect(libraryPhrase(page, texts[2])).toBeVisible()
  await phraseSearch(page).fill('')
  await page.getByRole('navigation').getByRole('button', { name: 'Practice', exact: true }).click()

  await setLimits(page, 1, 2)
  await press(page, 'Daily')
  await expect(modeLine(page)).toHaveText('Daily · 2 / 2 new')
  await meet(page, texts[1])
  await expect(fact(page, 'today-title')).toHaveText('Today — 2 new')

  let releaseUpload!: () => void
  const uploadHeld = new Promise<void>((resolve) => (releaseUpload = resolve))
  await page.route('**/api/recordings', async (route) => {
    if (route.request().method() === 'POST') await uploadHeld
    await route.continue()
  })
  await press(page, 'Extra')
  await expect(modeLine(page)).toHaveText('Extra · 3 / 4 new')
  await revealAnswer(page)
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
  await expect(fact(page, 'today-recordings')).toHaveText('1 recording made')
  await expect(fact(page, 'progress-recordings')).toHaveText('1 recording made')

  await press(page, 'Extra')
  await expect(modeLine(page)).toHaveText('Extra · 5 / 6 new')
  await meet(page, texts[4])
  await meet(page, texts[5])
  await expect(fact(page, 'today-title')).toHaveText('Today — 6 new')
  await expect(fact(page, 'library')).toHaveText('6 later')

  await press(page, 'See today’s phrases')
  const studied = facts(page).getByRole('list', { name: 'Today’s phrases' }).getByRole('listitem')
  await expect(studied).toHaveCount(5)
  await expectStudied(studied.first(), texts[0], 'Again · Daily · Small talk')
  await expectStudied(studied.nth(2), texts[2], 'Again · Extra · Small talk')
  await facts(page).getByRole('button', { name: 'More', exact: true }).click()
  await expect(studied).toHaveCount(6)
  await expect(facts(page).getByRole('button', { name: 'More', exact: true })).toHaveCount(0)

  await openDetails(page, texts[0])
  await details(page).getByRole('button', { name: 'Delete', exact: true }).click()
  await page
    .getByRole('dialog', { name: `Delete “${texts[0]}”?` })
    .getByRole('button', { name: 'Delete permanently' })
    .click()
  await page.getByRole('navigation').getByRole('button', { name: 'Practice', exact: true }).click()
  await press(page, 'See today’s phrases')
  await expectStudied(studied.first(), texts[0], 'Again · Daily · Small talk · deleted')
  await expect(fact(page, 'today-title')).toHaveText('Today — 6 new')
  await expect(fact(page, 'progress')).toHaveText('6 new phrases')
  await expect(fact(page, 'total-title')).toHaveText('Total — 5 phrases')
  await expect(fact(page, 'library')).toHaveText('5 later')

  await openDetails(page, texts[2])
  const history = details(page).locator('.take-history li')
  await details(page).getByRole('button', { name: 'Recording history' }).click()
  await expect(history).toHaveCount(1)
  await expect(history.locator('time')).toHaveText(/\d{4}-\d{2}-\d{2}/)
  const before = (await playedMedia(page)).length
  await history.getByRole('button', { name: 'Mine' }).click()
  await expect
    .poll(async () => (await playedMedia(page)).slice(before))
    .toEqual([expect.stringMatching(/\/api\/recordings\/.+\/audio$/)])
})

test('next-day entry keeps yesterday’s streak until practice and resets after a missed day', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, factsPackage, 6)
  await setLimits(page, 1, 1)
  await press(page, 'Daily')
  await meet(page, texts[0])
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')

  await moveClock(page, 24)
  await expect(fact(page, 'streak')).toBeVisible()
  await expect(fact(page, 'today-title')).toHaveCount(0)
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')
  await expect(fact(page, 'library')).toHaveText('1 due now · 5 new')
  await expect(page.getByTestId('lesson-counts')).toHaveText('1 due · 5 new')

  await press(page, 'Daily')
  await expect(modeLine(page)).toHaveText('Daily · 1 / 1 due')
  await card(page).getByRole('button', { name: 'Show' }).click()
  await press(page, 'Good')
  await meet(page, texts[1])
  await expect(fact(page, 'streak')).toHaveText('🔥 2-day practice streak')
  await expect(fact(page, 'today-title')).toHaveText('Today — 1 recall / 1 new')
  await expect(fact(page, 'today-ratings')).toHaveText('1 Good · 1 Again')

  await press(page, 'Extra')
  await meet(page, texts[2])
  await expect(page.getByText("Today's Extra practice is complete.", { exact: true })).toBeVisible()
  await expect(fact(page, 'today-title')).toHaveText('Today — 1 recall / 2 new')
  await page.getByRole('navigation').getByRole('button', { name: 'Library', exact: true }).click()
  await phraseSearch(page).fill(texts[3])
  await expect(libraryPhrase(page, texts[3])).toBeVisible()

  await page.getByRole('navigation').getByRole('button', { name: 'Practice', exact: true }).click()
  await moveClock(page, 72)
  await expect(fact(page, 'streak')).toHaveText('No current practice streak')
  await expect(fact(page, 'today-title')).toHaveCount(0)
  await expect(fact(page, 'best-streak')).toHaveText('Personal best: 2 days')
  await expect(fact(page, 'progress-title')).toHaveText('Progress — 2 days')
  await expect(fact(page, 'progress')).toHaveText('1 recall · 3 new phrases')
  await expect(fact(page, 'progress-ratings')).toHaveText('1 Good · 3 Again')
  await expect(fact(page, 'library')).toHaveText('3 due now · 3 new')
  await press(page, 'Daily')
  await expect(modeLine(page)).toHaveText('Daily · 1 / 1 due')
})

test('an open entry recomputes facts and counts when it returns on a new learning day', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, factsPackage, 6)
  await setLimits(page, 1, 1)
  await press(page, 'Daily')
  await meet(page, texts[0])
  const message = page.getByText("Today's Daily practice is complete.", { exact: true })
  await expect(message).toBeVisible()
  await expect(fact(page, 'today-title')).toHaveText('Today — 1 new')
  await expect(fact(page, 'library')).toHaveText('1 later · 5 new')
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
  await expect(fact(page, 'library')).toHaveText('1 due now · 5 new')
  await expect(fact(page, 'today-title')).toHaveCount(0)
  await expect(message).toHaveCount(0)
  await expect(studied).toHaveCount(0)
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')
  await expect(counts).toHaveText('1 due · 5 new')
  await importPackage(page, latePackage, 1)
  await expect(fact(page, 'streak')).toHaveText('🔥 1-day practice streak')
  await expect(message).toHaveCount(0)

  await press(page, 'Daily')
  await card(page).getByRole('button', { name: 'Show' }).click()
  await press(page, 'Good')
  await meet(page, texts[1])
  await expect(fact(page, 'today-title')).toHaveText('Today — 1 recall / 1 new')
  await press(page, 'See today’s phrases')
  await expect(studied).toHaveCount(2)
  await expectStudied(studied.first(), texts[0], 'Good · Daily · Small talk')
  await expectStudied(studied.last(), texts[1], 'Again · Daily · Small talk')
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
  await importPackage(page, pagedPackages[0]!, 20)
  await importPackage(page, pagedPackages[1]!, 1)
  await setLimits(page, 1, 6)
  await press(page, 'Daily')
  for (let step = 1; step <= 6; step++) {
    await expect(modeLine(page)).toHaveText(`Daily · ${step} / 6 new`)
    await rateNew(page)
  }
  await expect(fact(page, 'today-title')).toHaveText('Today — 6 new')
  await press(page, 'See today’s phrases')
  const studied = facts(page).getByRole('list', { name: 'Today’s phrases' }).getByRole('listitem')
  await expect(studied).toHaveCount(5)
  await facts(page).getByRole('button', { name: 'More', exact: true }).click()
  await expect(studied).toHaveCount(6)
  const lesson = page.getByRole('combobox', { name: 'Lesson', exact: true })
  const options = page.getByRole('option')
  await lesson.click()
  await expect(options).toHaveCount(21)
  await press(page, 'More lessons')
  await expect(options).toHaveCount(22)
  await options.filter({ hasText: 'Unit 21' }).click()
  await expect(lesson).toHaveValue('Unit 21 · Facts course')
  const counts = page.getByTestId('lesson-counts')
  await expect(counts).toHaveText('0 due · 1 new')

  const loads = count('/api/practice/facts')
  const lists = count('/api/practice/studied')
  const lessons = count('/api/lessons')
  await returnToPage(page)
  await expect.poll(() => count('/api/practice/facts')).toBe(loads + 2)
  await expect(lesson).toHaveValue('Unit 21 · Facts course')
  await expect(counts).toHaveText('0 due · 1 new')
  await lesson.click()
  await expect(options).toHaveCount(22)
  await expect(page.getByRole('button', { name: 'More lessons' })).toHaveCount(0)
  await lesson.press('Escape')
  await expect(lesson).toHaveValue('Unit 21 · Facts course')
  await expect(studied).toHaveCount(6)
  expect([count('/api/practice/studied'), count('/api/lessons')]).toEqual([lists, lessons])

  await setClock(page, 24)
  const newList = page.waitForRequest((request) =>
    request.url().includes('/api/practice/studied?learning_date=2026-11-03'),
  )
  const refreshed = page.waitForRequest((request) => {
    const url = new URL(request.url())
    return (
      url.pathname === '/api/lessons' &&
      url.searchParams.has('lesson_id') &&
      url.searchParams.get('offset') === '0' &&
      url.searchParams.get('limit') === '21'
    )
  })
  await returnToPage(page)
  await Promise.all([newList, refreshed])
  await expect(fact(page, 'library')).toHaveText('6 due now · 15 new')
  await expect(fact(page, 'total-title')).toHaveText('Total — 21 phrases')
  await expect(fact(page, 'today-title')).toHaveCount(0)
  await expect(studied).toHaveCount(0)
  await lesson.click()
  await expect(options).toHaveCount(22)
  await lesson.press('Escape')
  await expect(counts).toHaveText('0 due · 1 new')

  // The remounted picker loads only the first page, yet still names the later lesson.
  await page.getByRole('navigation').getByRole('button', { name: 'Library', exact: true }).click()
  await expect(lesson).toHaveValue('Unit 21 · Facts course')
  await expect(counts).toHaveText('0 due · 1 new')
  await lesson.click()
  await expect(options).toHaveCount(21)
})

test('a Daily finished just after midnight shows its own completion message', async ({ page }) => {
  await setClock(page, 13 + 50 / 60)
  await page.goto('/')
  await importPackage(page, factsPackage, 6)
  await setLimits(page, 1, 1)
  await press(page, 'Daily')
  await meet(page, texts[0])
  const message = page.getByText("Today's Daily practice is complete.", { exact: true })
  await expect(message).toBeVisible()
  await expect(fact(page, 'today-title')).toHaveText('Today — 1 new')

  await setClock(page, 14 + 10 / 60)
  await press(page, 'Daily')
  await expect(modeLine(page)).toHaveText('Daily · 1 / 1 new')
  await meet(page, texts[1])
  await expect(fact(page, 'today-title')).toHaveText('Today — 1 new')
  await expect(message).toBeVisible()
  await expect(fact(page, 'streak')).toHaveText('🔥 2-day practice streak')
})

test('the completion message stays visible when the entry facts cannot load', async ({ page }) => {
  await page.goto('/')
  await importPackage(page, factsPackage, 6)
  await setLimits(page, 1, 1)
  await press(page, 'Daily')
  await revealAnswer(page)
  await expect(card(page).getByRole('heading', { name: texts[0] })).toBeVisible()
  await page.route('**/api/practice/facts', (route) => route.fulfill({ status: 503, json: {} }))
  await rateNew(page)
  await expect(facts(page).getByRole('alert')).toHaveText('Error: Request failed (503)')
  await expect(page.getByText("Today's Daily practice is complete.", { exact: true })).toBeVisible()
})

test.describe('mobile', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true })

  test('mobile Today facts fit above the entry controls', async ({ page }) => {
    await page.goto('/')
    await importPackage(page, factsPackage, 6, true)
    await setLimits(page, 1, 1)
    await page.getByRole('button', { name: 'Daily', exact: true }).tap()
    await revealAnswer(page)
    await expect(card(page).getByRole('heading', { name: texts[0] })).toBeVisible()
    await rateNew(page, true)
    await expect(fact(page, 'today-title')).toHaveText('Today — 1 new')
    await facts(page).getByRole('button', { name: 'See today’s phrases' }).tap()
    await expect(facts(page).getByRole('listitem').last()).toContainText(texts[0])
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
    await page.getByRole('button', { name: 'Extra', exact: true }).tap()
    await expect(modeLine(page)).toHaveText('Extra · 2 / 2 new')
  })
})
