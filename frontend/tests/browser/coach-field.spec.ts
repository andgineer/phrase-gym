import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import {
  importStatus,
  libraryPhrase,
  playedMedia,
  rateNew,
  revealAnswer,
  savedTakes,
} from './helpers'

const typedPhrase = 'strongly believe — We strongly believe it works.'
const phraseText = 'We strongly believe it works.'
const typedWord = 'reluctant'
const wordText = 'My friend said reluctant at the meeting.'

const field = (page: Page) => page.getByRole('textbox', { name: 'Phrase or word for Coach' })
const coachButton = (page: Page) => page.getByRole('button', { name: 'Send', exact: true })
const confirmation = (page: Page, text: string) =>
  page.getByRole('status').filter({ hasText: text })

async function submitToCoach(page: Page, typed: string): Promise<string> {
  await page.getByRole('navigation').getByRole('button', { name: 'Coach', exact: true }).click()
  await field(page).fill(typed)
  const response = page.waitForResponse('**/api/coach/requests')
  await coachButton(page).click()
  const { request_id: requestId } = await (await response).json()
  await expect(confirmation(page, 'Saved for Coach.')).toBeVisible()
  await expect(field(page)).toHaveValue('')
  return requestId
}

async function awaitPublished(page: Page, requestId: string) {
  await expect
    .poll(() => importStatus(page, requestId), { timeout: 20_000 })
    .toMatchObject({ status: 'added', added_count: 1 })
}

test.beforeEach(async ({ page }) => installFakeMedia(page))

test('a typed phrase and word reach Added with Coach while practice continues', async ({
  page,
}) => {
  await page.goto('/')
  await page.getByRole('navigation').getByRole('button', { name: 'Coach', exact: true }).click()
  await expect(coachButton(page)).toBeDisabled()
  await field(page).fill('   ')
  await expect(coachButton(page)).toBeDisabled()
  await awaitPublished(page, await submitToCoach(page, typedPhrase))
  await page.getByRole('navigation').getByRole('button', { name: 'Practice' }).click()
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  const card = page.locator('article.practice-card')
  await expect(card.locator('.practice-prompt')).toContainText('__________')
  await card.getByRole('button', { name: 'Record a take' }).click()
  const stop = card.getByRole('button', { name: 'Stop recording' })
  await expect(stop).toBeVisible()

  const beforeLeave = (await (await page.request.get('/api/practice/current')).json()).card
  const wordRequest = await submitToCoach(page, typedWord)
  await expect(card).toHaveCount(0)
  await expect.poll(async () => (await savedTakes(page, phraseText)).length).toBe(1)
  await page.getByRole('navigation').getByRole('button', { name: 'Practice' }).click()
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  const resumed = (await (await page.request.get('/api/practice/current')).json()).card
  expect(resumed.view_token).toBe(beforeLeave.view_token)
  await revealAnswer(page)
  await expect(card.getByRole('heading', { name: phraseText })).toBeVisible()
  await card.getByRole('button', { name: 'Again', exact: true }).click()

  await awaitPublished(page, wordRequest)
  await page.getByRole('navigation').getByRole('button', { name: 'Practice' }).click()
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  await revealAnswer(page)
  await expect(card.getByRole('heading', { name: wordText })).toBeVisible()
  await rateNew(page)
  const lessons = await (await page.request.get('/api/lessons')).json()
  expect(lessons.items).toEqual([
    expect.objectContaining({ title: 'Added with Coach', phrase_count: 2 }),
  ])

  const tomorrow = new Date(Date.now() + 26 * 3600 * 1000).toISOString()
  await page.request.post(`/__test/clock/${tomorrow}`)
  await page.reload()
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  await expect(card.locator('.practice-prompt')).toContainText('__________')
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await revealAnswer(page)
  await expect(page.getByTestId('practice-phrase')).toHaveText(/^(We strongly|My friend said)/)
  await card.getByRole('button', { name: 'Good', exact: true }).click()

  await page.getByRole('navigation').getByRole('button', { name: 'Library', exact: true }).click()
  await libraryPhrase(page, phraseText).click()
  await page.getByRole('button', { name: 'Recording history' }).click()
  const [take] = await savedTakes(page, phraseText)
  const before = (await playedMedia(page)).length
  await page.getByRole('button', { name: 'Mine' }).click()
  await expect
    .poll(async () => (await playedMedia(page)).slice(before))
    .toEqual([expect.stringContaining(take.url)])
})
