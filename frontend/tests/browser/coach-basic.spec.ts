import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { importStatus, libraryPhrase, phraseSearch, playedMedia, revealAnswer } from './helpers'

test.use({ coachRoute: 'gemini' })

const sentence = 'We strongly believe this will work'
const tab = (page: Page, name: string) =>
  page.getByRole('navigation').getByRole('button', { name, exact: true })
const field = (page: Page) => page.getByRole('textbox', { name: 'Phrase or word for Coach' })
const results = (page: Page) =>
  page.getByRole('list', { name: 'Coach results' }).getByRole('listitem')

async function sendFromCoach(page: Page, typed: string, confirmation = 'Saved for Coach.') {
  await tab(page, 'Coach').click()
  await field(page).fill(typed)
  const response = page.waitForResponse('**/api/coach/requests')
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  const { request_id: requestId } = await (await response).json()
  await expect(page.getByRole('status').filter({ hasText: confirmation })).toBeVisible()
  return requestId as string
}

async function settled(page: Page, requestId: string, status: string) {
  await expect
    .poll(async () => (await importStatus(page, requestId)).status, { timeout: 20_000 })
    .toBe(status)
}

test.beforeEach(async ({ page }) => installFakeMedia(page))

test('a sent sentence becomes one basic card with its gap and matching reference', async ({
  page,
}) => {
  await page.goto('/')
  await settled(page, await sendFromCoach(page, sentence), 'added')
  await tab(page, 'Library').click()
  await expect(libraryPhrase(page, sentence)).toBeVisible()
  const lessons = await (await page.request.get('/api/lessons?language=en')).json()
  expect(lessons.items).toEqual([
    expect.objectContaining({ title: 'Added with Coach', phrase_count: 1 }),
  ])

  await tab(page, 'Practice').click()
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  const card = page.locator('article.practice-card')
  await expect(card.locator('.practice-instruction')).toHaveText('Произнесите пропущенное.')
  await expect(card.locator('.practice-prompt')).toHaveText('We __________ this will work')
  await expect(card).not.toContainText('strongly')
  await expect(card.getByRole('button', { name: 'Hint' })).toBeDisabled()
  await revealAnswer(page)
  await expect(page.getByTestId('practice-phrase')).toHaveText(sentence)
  await expect(card.getByRole('button', { name: /More explanation/ })).toHaveCount(0)
  const { card: current } = await (
    await page.request.get('/api/practice/current?language=en')
  ).json()
  const before = (await playedMedia(page)).length
  await card.getByRole('button', { name: 'Ref', exact: true }).click()
  await expect
    .poll(async () => (await playedMedia(page)).slice(before))
    .toContainEqual(
      expect.stringMatching(new RegExp(`/api/phrases/${current.phrase_id}/reference$`)),
    )

  await sendFromCoach(page, sentence, 'Already in your Library.')
  const again = await (await page.request.get('/api/lessons?language=en')).json()
  expect(again.items[0].phrase_count).toBe(1)
})

test('a phrase sent alone is recalled from a partial form of every word', async ({ page }) => {
  await page.goto('/')
  await settled(page, await sendFromCoach(page, 'by and large'), 'added')
  await tab(page, 'Practice').click()
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  const card = page.locator('article.practice-card')
  await expect(card.locator('.practice-instruction')).toHaveText(
    'Произнесите слово или фразу целиком.',
  )
  await expect(card.locator('.practice-prompt')).toHaveText('b_ a__ l___e')
  await expect(card).not.toContainText('large')
  await revealAnswer(page)
  await expect(page.getByTestId('practice-phrase')).toHaveText('by and large')
})

test('Coach and Library sends show corrections and rejections in the Coach inbox', async ({
  page,
}) => {
  await page.goto('/')
  await settled(page, await sendFromCoach(page, 'colour'), 'added')
  await settled(page, await sendFromCoach(page, 'definately'), 'added')
  await page.reload()
  await expect(results(page)).toHaveCount(1)
  await expect(results(page)).toContainText('definately → definitely')
  await expect(results(page)).toContainText('English · Corrected and added with Coach')

  await tab(page, 'Library').click()
  await phraseSearch(page).fill('Guten Morgen')
  const request = page.waitForResponse('**/api/coach/requests')
  await page.getByRole('button', { name: /Send “Guten Morgen” to Coach/ }).click()
  const { request_id: rejected } = await (await request).json()
  await settled(page, rejected, 'error')
  await tab(page, 'Coach').click()
  const rejection = results(page).filter({ hasText: 'Guten Morgen' })
  await expect(rejection).toContainText('English · Not added')
  await expect(rejection).toContainText('Это немецкий текст, а не английский.')

  await sendFromCoach(page, 'Guten Morgen', 'Already checked: Coach could not add this.')
  expect(await importStatus(page, rejected)).toMatchObject({ status: 'error' })
  await rejection.getByRole('button', { name: 'Dismiss “Guten Morgen”' }).click()
  await results(page)
    .filter({ hasText: 'definately' })
    .getByRole('button', { name: /Dismiss/ })
    .click()
  await expect(results(page)).toHaveCount(0)
  await page.reload()
  await expect(page.getByRole('list', { name: 'Coach results' })).toHaveCount(0)
  await tab(page, 'Practice').click()
  await expect(page.getByRole('list', { name: 'Preparation problems' })).toHaveCount(0)
})

test('a target too long for one card says so in the Coach inbox', async ({ page }) => {
  await page.goto('/')
  const typed = Array(30).fill('strongly').join(' ')
  await settled(page, await sendFromCoach(page, typed), 'error')
  await page.reload()
  const result = results(page).filter({ hasText: typed })
  await expect(result).toContainText('English · Not added')
  await expect(result).toContainText('Текст слишком длинный для одной карточки.')
  await expect(result.getByRole('button', { name: 'Retry', exact: true })).toHaveCount(0)
  await tab(page, 'Practice').click()
  await expect(page.getByRole('list', { name: 'Preparation problems' })).toHaveCount(0)
})

test.describe('with Coach offline', () => {
  test.use({ coachEnabled: false })

  test('a service failure gives a retryable Coach result and a Practice notice', async ({
    page,
  }) => {
    await page.goto('/')
    const requestId = await sendFromCoach(page, 'by and large')
    await page.request.post('/api/imports/coach/next')
    await page.request.post(`/api/imports/${requestId}/failure`, {
      data: { error: 'Gemini: Not logged into Antigravity — run: agy login' },
    })
    await tab(page, 'Practice').click()
    const notices = page.getByRole('list', { name: 'Preparation problems' }).getByRole('listitem')
    await expect(notices).toContainText(
      'Coach could not prepare “by and large”: Gemini: Not logged into Antigravity',
    )
    await tab(page, 'Coach').click()
    await expect(results(page)).toContainText('English · Coach could not finish')
    await results(page).getByRole('button', { name: 'Retry', exact: true }).click()
    await expect(results(page)).toHaveCount(0)
    expect(await importStatus(page, requestId)).toMatchObject({ status: 'pending', error: null })
    await tab(page, 'Practice').click()
    await expect(page.getByRole('list', { name: 'Preparation problems' })).toHaveCount(0)
  })
})
