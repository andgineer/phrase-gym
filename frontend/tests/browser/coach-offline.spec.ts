import { expect, test } from './fixtures'
import { importStatus, silentReference } from './helpers'

test.use({ coachEnabled: false })

const typed = 'by and large'

const card = (packageId: string) => ({
  item_id: packageId,
  text: 'By and large, the plan worked.',
  explanation: '**by and large** means “mostly” or “in general”.',
  cues: [
    { situation: '__________, the trip went well. (=On the whole)' },
    { situation: '__________, people liked the change. (=Mostly)' },
  ],
  recall_hints: ['by / la...', 'and / large'],
  usage_contexts: ['summing up'],
})

test('typed text waits for offline Coach; failures arrive with the facts and can be retried', async ({
  page,
}) => {
  const statusReads: string[] = []
  page.on('request', (request) => {
    if (request.method() === 'GET' && new URL(request.url()).pathname.startsWith('/api/imports'))
      statusReads.push(request.url())
  })
  await page.clock.install()
  await page.goto('/')
  await expect(page.getByText('Add your first phrase', { exact: true })).toBeVisible()
  const field = page.getByRole('textbox', { name: 'Phrase or word for Coach' })
  const coach = page.getByRole('button', { name: 'Coach', exact: true })
  await field.fill(typed)
  const response = page.waitForResponse('**/api/coach/requests')
  await field.press('Enter')
  const { request_id: requestId } = await (await response).json()
  const saved = page.getByRole('status').filter({ hasText: 'Saved for Coach.' })
  await expect(saved).toBeVisible()
  await expect(field).toHaveValue('')
  await page.clock.runFor(5_000)
  await expect(saved).toHaveCount(0)
  await field.fill(typed.toUpperCase())
  await coach.click()
  await expect(
    page.getByRole('status').filter({ hasText: 'Already waiting for Coach.' }),
  ).toBeVisible()
  await page.clock.runFor(60_000)
  expect(await importStatus(page, requestId)).toMatchObject({ status: 'pending' })

  await page.request.post('/api/imports/coach/next')
  await page.request.post(`/api/imports/${requestId}/failure`, {
    data: { error: 'Codex is not logged in' },
  })
  await page.reload()
  const notices = page.getByRole('list', { name: 'Preparation problems' }).getByRole('listitem')
  await expect(notices).toHaveCount(1)
  await expect(notices).toContainText(
    'Coach could not prepare “by and large”: Codex is not logged in',
  )
  await expect(notices).toContainText('Type the same text into the Coach field to retry.')
  await page.getByRole('button', { name: `Dismiss notice about ${typed}` }).click()
  await expect(notices).toHaveCount(0)
  const facts = await (await page.request.get('/api/practice/facts')).json()
  expect(facts.notices).toEqual([])

  await field.fill(typed)
  await coach.click()
  await expect(saved).toBeVisible()
  expect(await importStatus(page, requestId)).toMatchObject({ status: 'pending', error: null })
  await page.request.post('/api/imports/coach/next')
  await page.request.post(`/api/imports/${requestId}/result`, {
    data: { result_id: requestId, phrase: card(requestId), reference: silentReference() },
  })
  await page.getByRole('navigation').getByRole('button', { name: 'Practice' }).click()
  await expect(page.getByTestId('lesson-counts')).toHaveText('0 due · 1 new')
  await field.fill(typed)
  await coach.click()
  await expect(
    page.getByRole('status').filter({ hasText: 'Already in your Library.' }),
  ).toBeVisible()
  expect(statusReads).toEqual([])
})

test('a content failure has no retry line and bold help renders without asterisks', async ({
  page,
}) => {
  await page.goto('/')
  const field = page.getByRole('textbox', { name: 'Phrase or word for Coach' })
  const coach = page.getByRole('button', { name: 'Coach', exact: true })
  for (const text of ['https://example.com/lesson', typed]) {
    await field.fill(text)
    await coach.click()
    await expect(page.getByRole('status').filter({ hasText: 'Saved for Coach.' })).toBeVisible()
  }
  const url = (await (await page.request.post('/api/imports/coach/next')).json()).package_id
  await page.request.post(`/api/imports/${url}/failure`, {
    data: { error: 'This is a web address, not an English word.', retry_helps: false },
  })
  const phrase = (await (await page.request.post('/api/imports/coach/next')).json()).package_id
  await page.request.post(`/api/imports/${phrase}/result`, {
    data: { result_id: phrase, phrase: card(phrase), reference: silentReference() },
  })
  await page.reload()
  const notices = page.getByRole('list', { name: 'Preparation problems' }).getByRole('listitem')
  await expect(notices).toContainText('This is a web address, not an English word.')
  await expect(notices).not.toContainText('Type the same text into the Coach field to retry.')

  await page.getByRole('navigation').getByRole('button', { name: 'Library', exact: true }).click()
  await page.getByRole('list', { name: 'Phrases' }).getByRole('button').first().click()
  const details = page.locator('section.phrase-details')
  await expect(details.locator('.answer-help strong')).toHaveText('by and large')
  await expect(details.locator('.answer-help')).not.toContainText('**')
})
