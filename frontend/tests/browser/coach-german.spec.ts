import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { importStatus, libraryPhrase, phraseSearch, playedMedia, revealAnswer } from './helpers'

const field = (page: Page) => page.getByRole('textbox', { name: 'Phrase or word for Coach' })
const languageButton = (page: Page, name: string) =>
  page.getByRole('navigation', { name: 'Learning language' }).getByRole('button', { name })
const results = (page: Page) =>
  page.getByRole('list', { name: 'Coach results' }).getByRole('listitem')

test.beforeEach(async ({ page }) => installFakeMedia(page))

async function send(page: Page, address: string, typed: string) {
  await page.goto(address)
  await field(page).fill(typed)
  const response = page.waitForResponse('**/api/coach/requests')
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  const { request_id: requestId } = await (await response).json()
  await expect(page.getByRole('status').filter({ hasText: 'Saved for Coach.' })).toBeVisible()
  return requestId as string
}

async function settled(page: Page, requestId: string, status: string) {
  await expect
    .poll(async () => (await importStatus(page, requestId)).status, { timeout: 20_000 })
    .toBe(status)
}

async function startGerman(page: Page) {
  await page.goto('/#de')
  await expect(languageButton(page, 'German')).toHaveAttribute('aria-current', 'true')
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  return page.locator('article.practice-card')
}

async function expectGermanReference(page: Page) {
  const card = page.locator('article.practice-card')
  const { card: current } = await (
    await page.request.get('/api/practice/current?language=de')
  ).json()
  const before = (await playedMedia(page)).length
  await card.getByRole('button', { name: 'Ref', exact: true }).click()
  await expect
    .poll(async () => (await playedMedia(page)).slice(before))
    .toContainEqual(
      expect.stringMatching(new RegExp(`/api/phrases/${current.phrase_id}/reference$`)),
    )
}

test('a German word sent from Coach becomes a German card with Russian meaning', async ({
  page,
}) => {
  await settled(page, await send(page, '/#de/coach', 'Tisch'), 'added')
  await page.goto('/#de/library')
  await expect(libraryPhrase(page, 'Das ist Tisch.')).toBeVisible()
  await page.goto('/#en/library')
  await expect(libraryPhrase(page, 'Das ist Tisch.')).toHaveCount(0)

  const card = await startGerman(page)
  await expect(card.locator('.practice-prompt')).toContainText('(=значение')
  await expect(card).not.toContainText('Tisch')
  await revealAnswer(page)
  const answer = page.getByTestId('practice-phrase')
  await expect(answer).toHaveText('Das ist Tisch.')
  await expect(answer).toHaveAttribute('lang', 'de')
  await expect(card).toContainText('Цель: Tisch.')
  await expectGermanReference(page)
})

test.describe('on the free route', () => {
  test.use({ coachRoute: 'gemini' })

  test('a German word sent from a Library search asks for the whole word in Russian', async ({
    page,
  }) => {
    await page.goto('/#de/library')
    await phraseSearch(page).fill('Tisch')
    const response = page.waitForResponse('**/api/coach/requests')
    await page.getByRole('button', { name: 'Send “Tisch” to Coach' }).click()
    const { request_id: requestId } = await (await response).json()
    await settled(page, requestId, 'added')
    await page.reload()
    await expect(libraryPhrase(page, 'Tisch')).toBeVisible()

    const card = await startGerman(page)
    await expect(card.locator('.practice-instruction')).toHaveText(
      'Произнесите слово или фразу целиком.',
    )
    await expect(card.locator('.practice-prompt')).toHaveText('T___h')
    await revealAnswer(page)
    await expect(page.getByTestId('practice-phrase')).toHaveText('Tisch')
    await expectGermanReference(page)
  })

  test('a corrected German card names the correction in Russian', async ({ page }) => {
    await settled(page, await send(page, '/#de/coach', 'die tür'), 'added')
    const card = await startGerman(page)
    await expect(card.locator('.practice-prompt')).toHaveText('d__ T__')
    await revealAnswer(page)
    await expect(page.getByTestId('practice-phrase')).toHaveText('die Tür')
    await expect(card).toContainText('Исправлено, было: «die tür».')
  })
})

test.describe('without a German voice', () => {
  test.use({ coachVoices: ['en'] })

  test('a German send fails for retry while English cards keep coming', async ({ page }) => {
    await settled(page, await send(page, '/#de/coach', 'Tisch'), 'error')
    await page.reload()
    const failure = results(page).filter({ hasText: 'Tisch' })
    await expect(failure).toContainText('German · Coach could not finish')
    await expect(failure).toContainText('У Coach пока нет голоса для этого языка.')
    await expect(failure.getByRole('button', { name: 'Retry' })).toBeVisible()

    await settled(page, await send(page, '/#en/coach', 'reluctant'), 'added')
    await page.goto('/#en/library')
    await expect(libraryPhrase(page, 'She felt reluctant today.')).toBeVisible()
  })
})
