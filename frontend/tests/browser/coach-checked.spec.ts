import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { importStatus, playedMedia, revealAnswer } from './helpers'

test.use({ coachRoute: 'claude' })

const tab = (page: Page, name: string) =>
  page.getByRole('navigation').getByRole('button', { name, exact: true })
const results = (page: Page) =>
  page.getByRole('list', { name: 'Coach results' }).getByRole('listitem')

async function send(page: Page, typed: string, status: string) {
  await tab(page, 'Coach').click()
  await page.getByRole('textbox', { name: 'Phrase or word for Coach' }).fill(typed)
  const response = page.waitForResponse('**/api/coach/requests')
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  const { request_id: requestId } = await (await response).json()
  await expect
    .poll(async () => (await importStatus(page, requestId)).status, { timeout: 20_000 })
    .toBe(status)
}

async function start(page: Page) {
  await tab(page, 'Practice').click()
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  return page.locator('article.practice-card')
}

test.beforeEach(async ({ page }) => installFakeMedia(page))

test('a checked example reaches practice with its gap, meaning cue, hint and reference', async ({
  page,
}) => {
  const sentence = 'We strongly believe it works.'
  await page.goto('/')
  await send(page, `strongly believe — ${sentence}`, 'added')
  const card = await start(page)
  await expect(card.locator('.practice-prompt')).toHaveText(
    /^(We __________ it works\. \(=sense 1\)|My friend said __________ at the meeting\. \(=sense 2\))$/,
  )
  await expect(card.locator('.practice-instruction')).toHaveCount(0)
  await expect(card).not.toContainText('strongly')
  await card.getByRole('button', { name: 'Hint', exact: true }).click()
  await expect(
    card.getByText(/^Hint: (stro\.\.\. \/ morning|str\.\.\. \/ garden|stro\.\.\. \/ station)$/),
  ).toBeVisible()
  await revealAnswer(page)
  await expect(page.getByTestId('practice-phrase')).toHaveText(sentence)
  await expect(card).toContainText('Target: strongly believe. A useful way to say it.')
  const { card: current } = await (await page.request.get('/api/practice/current')).json()
  const before = (await playedMedia(page)).length
  await card.getByRole('button', { name: 'Ref', exact: true }).click()
  await expect
    .poll(async () => (await playedMedia(page)).slice(before))
    .toContainEqual(
      expect.stringMatching(new RegExp(`/api/phrases/${current.phrase_id}/reference$`)),
    )
})

test('a doubted addition leaves basic practice and a disputed form ends in Coach', async ({
  page,
}) => {
  const sentence = 'We strongly believe it works, unsafe-meaning.'
  await page.goto('/')
  await send(page, `strongly believe — ${sentence}`, 'added')
  const card = await start(page)
  await expect(card.locator('.practice-instruction')).toHaveText('Say the missing wording.')
  await expect(card.locator('.practice-prompt')).toHaveText(
    'We __________ it works, unsafe-meaning.',
  )
  await expect(card.getByRole('button', { name: 'Hint' })).toBeDisabled()
  await revealAnswer(page)
  await expect(page.getByTestId('practice-phrase')).toHaveText(sentence)

  const disputed = 'strongly believe — We strongly believe it works, check-target.'
  await send(page, disputed, 'error')
  await page.reload()
  await tab(page, 'Coach').click()
  const result = results(page).filter({ hasText: 'check-target' })
  await expect(result).toContainText('English · Not added')
  await expect(result).toContainText('Coach could not confirm one form for this input.')
  await tab(page, 'Practice').click()
  await expect(page.getByRole('list', { name: 'Preparation problems' })).toHaveCount(0)
})
