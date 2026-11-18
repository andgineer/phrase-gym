import type { Locator, Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { card, modeLine } from './extra-package'
import { finishDaily, setSettings } from './practice-api'
import { rateNew, revealAnswer, importPackage, publishPackage } from './helpers'

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
  const refreshed = page.waitForResponse('**/api/practice/today*')
  await page.evaluate(() => window.dispatchEvent(new Event('pageshow')))
  await refreshed
}

async function meet(page: Page) {
  await revealAnswer(page)
  const text = await card(page).getByTestId('practice-phrase').innerText()
  await rateNew(page)
  return text
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

test.beforeEach(async ({ page }) => {
  await installFakeMedia(page)
  await setClock(page, 0)
})

test('Extra completion refreshes facts and pending uploads, and studied history pages', async ({
  page,
}) => {
  await publishPackage(page, factsPackage, 6)
  await setSettings(page, { daily_due_limit: 1, daily_new_limit: 5 })
  const completed = await finishDaily(page)
  const met = completed.map((card) => card.answer!.text)
  await page.goto('/')
  await expect(fact(page, 'today-title')).toHaveText('Today — 5 new')
  const panel = await facts(page).boundingBox()
  const daily = await page.getByRole('button', { name: 'Start', exact: true }).boundingBox()
  const field = await page.getByRole('combobox', { name: 'Lesson', exact: true }).boundingBox()
  expect(field!.y + field!.height).toBeLessThan(daily!.y)
  expect(panel!.y).toBeGreaterThan(field!.y + field!.height)

  let releaseUpload!: () => void
  const uploadHeld = new Promise<void>((resolve) => (releaseUpload = resolve))
  await page.route('**/api/recordings', async (route) => {
    if (route.request().method() === 'POST') await uploadHeld
    await route.continue()
  })
  await press(page, 'Extra')
  await expect(modeLine(page)).toHaveText('Extra · 6 / 10 new')
  await revealAnswer(page)
  met.push(await card(page).getByTestId('practice-phrase').innerText())
  await card(page).getByRole('button', { name: 'Record a take' }).click()
  await expect(card(page).getByRole('button', { name: 'Stop recording' })).toBeVisible()
  await rateNew(page)
  await expect(
    page.getByText('Nothing waiting today. Next due: 2026-11-03.', { exact: true }),
  ).toBeVisible()
  await expect(fact(page, 'pending-uploads')).toHaveText(
    '1 recording waiting to upload from this device',
  )
  await expect(fact(page, 'today-recordings')).toHaveCount(0)
  releaseUpload()
  await expect(fact(page, 'pending-uploads')).toHaveCount(0)
  await expect(fact(page, 'today-recordings')).toHaveText('1 recording made')
  await expect(fact(page, 'progress-recordings')).toHaveText('1 recording made')
  await expect(fact(page, 'today-title')).toHaveText('Today — 6 new')

  await press(page, 'See today’s phrases')
  const studied = facts(page).getByRole('list', { name: 'Today’s phrases' }).getByRole('listitem')
  await expect(studied).toHaveCount(5)
  await expectStudied(studied.first(), met[0]!, 'Again · Daily · Small talk')
  await facts(page).getByRole('button', { name: 'More', exact: true }).click()
  await expect(studied).toHaveCount(6)
  await expectStudied(studied.last(), met[5]!, 'Again · Extra · Small talk')
  await expect(facts(page).getByRole('button', { name: 'More', exact: true })).toHaveCount(0)
})

test('an open entry recomputes facts and counts when it returns on a new learning day', async ({
  page,
}) => {
  await page.goto('/')
  await importPackage(page, factsPackage, 6)
  await setLimits(page, 1, 1)
  await press(page, 'Start')
  await meet(page)
  const message = page.getByText("Today's practice is complete.", { exact: true })
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
})

test('returning on the same day keeps loaded pages and a later lesson; a new day resets them', async ({
  page,
}) => {
  const requests: string[] = []
  page.on('request', (request) =>
    requests.push(new URL(request.url()).pathname + new URL(request.url()).search),
  )
  const count = (prefix: string) => requests.filter((path) => path.startsWith(prefix)).length
  await publishPackage(page, pagedPackages[0]!, 20)
  await setSettings(page, { daily_due_limit: 1, daily_new_limit: 6 })
  await finishDaily(page)
  await page.goto('/')
  // Daily draws new phrases at random, so the later lesson arrives once Daily is done.
  await importPackage(page, pagedPackages[1]!, 1)
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
  await press(page, 'Start')
  await meet(page)
  const message = page.getByText("Today's practice is complete.", { exact: true })
  await expect(message).toBeVisible()
  await expect(fact(page, 'today-title')).toHaveText('Today — 1 new')

  await setClock(page, 14 + 10 / 60)
  await page.evaluate(() => window.dispatchEvent(new Event('pageshow')))
  await press(page, 'Start')
  await expect(modeLine(page)).toHaveText('1 / 1 new')
  await meet(page)
  await expect(fact(page, 'today-title')).toHaveText('Today — 1 new')
  await expect(message).toBeVisible()
  await expect(fact(page, 'streak')).toHaveText('🔥 2-day practice streak')
})

test('the completion message stays visible when the entry facts cannot load', async ({ page }) => {
  await page.goto('/')
  await importPackage(page, factsPackage, 6)
  await setLimits(page, 1, 1)
  await press(page, 'Start')
  await revealAnswer(page)
  await expect(card(page).getByTestId('practice-phrase')).toBeVisible()
  await page.route('**/api/practice/facts', (route) => route.fulfill({ status: 503, json: {} }))
  await rateNew(page)
  await expect(facts(page).getByRole('alert')).toHaveText('Error: Request failed (503)')
  await expect(page.getByText("Today's practice is complete.", { exact: true })).toBeVisible()
})

test.describe('mobile', () => {
  test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true })

  test('mobile Daily and Extra open above Today facts without scrolling', async ({ page }) => {
    await publishPackage(page, factsPackage, 6)
    await setSettings(page, { daily_due_limit: 1, daily_new_limit: 1 })
    const [completed] = await finishDaily(page)
    const drawn = completed!.answer!.text
    await page.goto('/')
    await expect(fact(page, 'today-title')).toHaveText('Today — 1 new')
    expect(await page.evaluate(() => window.scrollY)).toBe(0)
    const panel = await facts(page).boundingBox()
    for (const name of ['Start', 'Extra']) {
      const button = page.getByRole('button', { name, exact: true })
      await expect(button).toBeInViewport({ ratio: 1 })
      const box = await button.boundingBox()
      expect(box!.y + box!.height).toBeLessThan(panel!.y)
    }
    await facts(page).getByRole('button', { name: 'See today’s phrases' }).tap()
    await expect(facts(page).getByRole('listitem').last()).toContainText(drawn)
    expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
    await page.getByRole('button', { name: 'Extra', exact: true }).tap()
    await expect(modeLine(page)).toHaveText('Extra · 2 / 2 new')
  })
})
