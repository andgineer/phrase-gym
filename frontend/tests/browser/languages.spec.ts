import type { Page } from '@playwright/test'
import { expect, test } from './fixtures'
import { extraPackage, texts as englishTexts } from './extra-package'
import { publishPackage, rateNew, revealAnswer } from './helpers'
import { finishDaily, setSettings } from './practice-api'

test.use({ coachEnabled: false })

const germanTexts = [
  'Das ist mein Tisch.',
  'Ich rufe dich morgen an.',
  'Guten Morgen!',
  'Wie geht’s?',
]
const germanPackage = {
  format: 'phrase-gym.package',
  version: 3,
  package_id: 'deutsch-a1-v1',
  source_id: 'deutsch-a1',
  source_label: 'Deutschkurs A1',
  learning_language: 'de',
  meaning_language: 'ru',
  lessons: [
    {
      lesson_id: 'alltag',
      title: 'Alltag',
      phrases: germanTexts.map((text, index) => ({
        item_id: `alltag-${index}`,
        text,
        explanation: `Перевод: ${text}`,
        cues: [{ situation: `Ситуация номер ${index + 1}` }],
        recall_hints: [`hinweis ${index}`, `tipp ${index}`],
        usage_contexts: ['Alltag'],
      })),
    },
  ],
}

const languageButton = (page: Page, name: string) =>
  page.getByRole('navigation', { name: 'Learning language' }).getByRole('button', { name })
const tab = (page: Page, name: string) =>
  page.getByRole('navigation', { name: 'Pages' }).getByRole('button', { name, exact: true })
const counts = (page: Page) => page.locator('.waiting-counts')
const libraryTexts = (page: Page) =>
  page.getByRole('list', { name: 'Phrases' }).locator('strong[id^="phrase-result-"]')
const historyLength = (page: Page) => page.evaluate(() => window.history.length)
const hash = (page: Page) => page.evaluate(() => window.location.hash)

async function seedBoth(page: Page) {
  await publishPackage(page, extraPackage, englishTexts.length)
  await publishPackage(page, germanPackage, germanTexts.length)
}

async function expectGermanPractice(page: Page) {
  await expect(languageButton(page, 'German')).toHaveAttribute('aria-current', 'true')
  await expect(languageButton(page, 'English')).not.toHaveAttribute('aria-current')
  await expect(tab(page, 'Practice')).toHaveAttribute('aria-current', 'page')
  await expect(counts(page)).toContainText(`${germanTexts.length} new`)
}

test('bookmarked German pages open German and keep it on reload', async ({ page }) => {
  await page.goto('/')
  await seedBoth(page)
  await page.goto('/#de')
  await expectGermanPractice(page)
  await page.reload()
  await expectGermanPractice(page)

  await page.goto('/#de/library')
  await expect(tab(page, 'Library')).toHaveAttribute('aria-current', 'page')
  await expect(libraryTexts(page)).toHaveText(germanTexts)
  await expect(libraryTexts(page).first()).toHaveAttribute('lang', 'de')
  await page.reload()
  await expect(libraryTexts(page)).toHaveText(germanTexts)

  await page.goto('/#de/coach')
  await expect(tab(page, 'Coach')).toHaveAttribute('aria-current', 'page')
  await page.reload()
  await expect(tab(page, 'Coach')).toHaveAttribute('aria-current', 'page')
  expect(await hash(page)).toBe('#de/coach')
  await tab(page, 'Practice').click()
  expect(await hash(page)).toBe('#de')
  await expectGermanPractice(page)
})

test('addresses without a language open the English pages', async ({ page }) => {
  await page.goto('/')
  await seedBoth(page)
  await page.goto('/')
  await expect(languageButton(page, 'English')).toHaveAttribute('aria-current', 'true')
  await expect(counts(page)).toContainText(`${englishTexts.length} new`)
  await page.goto('/#library')
  await expect(tab(page, 'Library')).toHaveAttribute('aria-current', 'page')
  await expect(libraryTexts(page)).toHaveText(englishTexts)
  await page.goto('/#coach')
  await expect(tab(page, 'Coach')).toHaveAttribute('aria-current', 'page')
  await page.getByRole('textbox', { name: 'Phrase or word for Coach' }).fill('by and large')
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  await expect(page.getByRole('status').filter({ hasText: 'Saved for Coach.' })).toBeVisible()
  const work = await (await page.request.post('/api/imports/coach/next')).json()
  expect(work).toMatchObject({ text: 'by and large', learning_language: 'en' })
  await page.goto('/#settings')
  await expect(tab(page, 'Settings')).toHaveAttribute('aria-current', 'page')
  await expect(page.getByRole('navigation', { name: 'Learning language' })).toHaveCount(0)
  await page.goto('/#fr/library')
  await expect(tab(page, 'Practice')).toHaveAttribute('aria-current', 'page')
  await expect(languageButton(page, 'English')).toHaveAttribute('aria-current', 'true')
})

test('language buttons switch in place and Daily offers only that language', async ({ page }) => {
  await page.goto('/')
  await seedBoth(page)
  await page.goto('/#en')
  await expect(counts(page)).toContainText(`${englishTexts.length} new`)
  const before = await historyLength(page)
  await languageButton(page, 'German').click()
  expect(await hash(page)).toBe('#de')
  await expectGermanPractice(page)
  await languageButton(page, 'English').click()
  expect(await hash(page)).toBe('#en')
  await expect(counts(page)).toContainText(`${englishTexts.length} new`)
  await languageButton(page, 'German').click()
  expect(await historyLength(page)).toBe(before)

  await page.getByRole('button', { name: 'Start', exact: true }).click()
  expect(await hash(page)).toBe('#de')
  await revealAnswer(page)
  const answer = page.getByTestId('practice-phrase')
  await expect(answer).toHaveAttribute('lang', 'de')
  expect(germanTexts).toContain((await answer.textContent())?.trim())
  await rateNew(page)
  await page.reload()
  await expect(languageButton(page, 'German')).toHaveAttribute('aria-current', 'true')
  await expect(counts(page)).toContainText(`${germanTexts.length - 1} new`)
  await languageButton(page, 'English').click()
  await expect(counts(page)).toContainText(`${englishTexts.length} new`)
  await tab(page, 'Library').click()
  expect(await hash(page)).toBe('#en/library')
  await expect(libraryTexts(page)).toHaveText(englishTexts)
})

test('Coach and Library sends take the language of the page', async ({ page }) => {
  await page.goto('/#de')
  await page.getByRole('link', { name: 'Coach', exact: true }).click()
  await expect(tab(page, 'Coach')).toHaveAttribute('aria-current', 'page')
  expect(await hash(page)).toBe('#de/coach')
  const field = page.getByRole('textbox', { name: 'Phrase or word for Coach' })
  const saved = page.getByRole('status').filter({ hasText: 'Saved for Coach.' })
  await field.fill('Tisch')
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  await expect(saved).toBeVisible()

  await page.goto('/#de/library')
  await page.getByRole('searchbox', { name: 'Search phrases or concepts' }).fill('Stuhl')
  await page.getByRole('button', { name: 'Send “Stuhl” to Coach' }).click()
  await expect(saved).toBeVisible()

  await page.goto('/#en/coach')
  await field.fill('Morgen')
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  await expect(saved).toBeVisible()

  const claimed = []
  // A claimed request stays first until it finishes, so each one is failed after its claim.
  for (let index = 0; index < 3; index++) {
    const work = await (await page.request.post('/api/imports/coach/next')).json()
    claimed.push(work)
    await page.request.post(`/api/imports/${work.package_id}/failure`, {
      data: { error: 'Test finished', retry_helps: true },
    })
  }
  expect(claimed.map(({ text, learning_language }) => [text, learning_language])).toEqual([
    ['Tisch', 'de'],
    ['Stuhl', 'de'],
    ['Morgen', 'en'],
  ])
  expect(claimed[0].translation).toBe(true)
  expect(claimed[2].translation).toBe(false)
})

test('Today and Total follow the page language while Progress counts both', async ({ page }) => {
  await page.goto('/')
  await seedBoth(page)
  await finishDaily(page, 'en')
  await page.goto('/#en')
  const facts = page.getByRole('region', { name: 'Practice facts' })
  await expect(facts.getByTestId('today-title')).toHaveText(`Today — ${englishTexts.length} new`)
  await expect(facts.getByTestId('total-title')).toHaveText(
    `Total — ${englishTexts.length} phrases`,
  )
  await expect(facts.getByTestId('progress-title')).toHaveText('Progress — 1 day')
  await expect(facts.getByTestId('streak')).toHaveText('🔥 1-day practice streak')

  await languageButton(page, 'German').click()
  await expect(facts.getByTestId('total-title')).toHaveText(`Total — ${germanTexts.length} phrases`)
  await expect(facts.getByTestId('today-title')).toHaveCount(0)
  await expect(facts.getByTestId('progress-title')).toHaveText('Progress — 1 day')
  await expect(facts.getByTestId('streak')).toHaveText('🔥 1-day practice streak')
})

test('Settings edits the native language and each translation choice', async ({ page }) => {
  await page.goto('/#de/settings')
  const english = page.getByRole('checkbox', { name: 'English' })
  const german = page.getByRole('checkbox', { name: 'German' })
  await expect(english).not.toBeChecked()
  await expect(german).toBeChecked()
  await english.check()
  await page.getByRole('button', { name: 'Save settings' }).click()
  await expect(page.getByRole('status').filter({ hasText: 'Settings saved.' })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('checkbox', { name: 'English' })).toBeChecked()
  const settings = await (await page.request.get('/api/settings')).json()
  expect(settings.translation).toEqual({ en: true, de: true })
})

test.describe('with Coach running', () => {
  test.use({ coachEnabled: true })

  test('a German send waits for a German voice as a retryable failure', async ({ page }) => {
    await page.goto('/#de/coach')
    await page.getByRole('textbox', { name: 'Phrase or word for Coach' }).fill('Tisch')
    await page.getByRole('button', { name: 'Send', exact: true }).click()
    await expect(page.getByRole('status').filter({ hasText: 'Saved for Coach.' })).toBeVisible()
    const results = page.getByRole('list', { name: 'Coach results' })
    await expect
      .poll(async () => {
        await page.reload()
        return results.count()
      })
      .toBe(1)
    await expect(results).toContainText('German · Coach could not finish')
    await expect(results).toContainText('У Coach пока нет голоса для этого языка.')
    await expect(results.getByRole('button', { name: 'Retry' })).toBeVisible()
  })
})

const changeHash = (page: Page, next: string) =>
  page.evaluate((value) => (window.location.hash = value), next)

test('an address changed while the Library is open shows the new language', async ({ page }) => {
  await page.goto('/')
  await seedBoth(page)
  await page.goto('/#en/library')
  await expect(libraryTexts(page)).toHaveText(englishTexts)
  await changeHash(page, '#de/library')
  await expect(tab(page, 'Library')).toHaveAttribute('aria-current', 'page')
  await expect(libraryTexts(page)).toHaveText(germanTexts)
  await page.getByRole('searchbox', { name: 'Search phrases or concepts' }).fill('Stuhl')
  await page.getByRole('button', { name: 'Send “Stuhl” to Coach' }).click()
  await expect(page.getByRole('status').filter({ hasText: 'Saved for Coach.' })).toBeVisible()
  const work = await (await page.request.post('/api/imports/coach/next')).json()
  expect(work).toMatchObject({ text: 'Stuhl', learning_language: 'de' })
})

test('an address changed while Coach is open sends in the new language', async ({ page }) => {
  await page.goto('/#en/coach')
  const field = page.getByRole('textbox', { name: 'Phrase or word for Coach' })
  await field.fill('draft')
  await changeHash(page, '#de/coach')
  // The remounted page starts empty, which shows it now belongs to German.
  await expect(field).toHaveValue('')
  await field.fill('Tisch')
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  await expect(page.getByRole('status').filter({ hasText: 'Saved for Coach.' })).toBeVisible()
  const work = await (await page.request.post('/api/imports/coach/next')).json()
  expect(work).toMatchObject({ text: 'Tisch', learning_language: 'de' })
})

test('a failed English Extra cannot be retried from the German page', async ({ page }) => {
  await page.goto('/')
  await seedBoth(page)
  await setSettings(page, { daily_due_limit: 1, daily_new_limit: 1 })
  await finishDaily(page, 'en')
  await page.goto('/#en')
  await page.route('**/api/practice/extra', (route) => route.fulfill({ status: 503, json: {} }))
  await page.getByRole('button', { name: 'Extra', exact: true }).click()
  const retry = page.getByRole('button', { name: 'Retry', exact: true })
  await expect(retry).toBeVisible()
  await languageButton(page, 'German').click()
  await expect(languageButton(page, 'German')).toHaveAttribute('aria-current', 'true')
  await expect(counts(page)).toContainText('1 new')
  await expect(retry).toHaveCount(0)
  await expect(page.getByText('Extra was not confirmed')).toHaveCount(0)
})
