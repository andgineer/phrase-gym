import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('silent session completes without a microphone and keeps one rating per target', async ({
  page,
  request,
}) => {
  await page.addInitScript(() => {
    navigator.mediaDevices.getUserMedia = async () => {
      throw new Error('Silent self-check requested a microphone')
    }
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.locator('.practice-card')).toBeVisible()
  await expect(page.getByTestId('practice-phrase')).toContainText('_____')
  const sessionId = page.url().split('/').pop()
  for (const name of ['Recalled', 'With effort', 'Recalled']) {
    await expect(page.getByRole('button', { name, exact: false })).toBeEnabled()
    await page.getByRole('button', { name, exact: false }).click()
  }
  await expect(page.getByRole('heading', { name: 'Session complete' })).toBeVisible()
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  expect(session.attempts).toHaveLength(3)
  expect(
    session.attempts.every(
      (a: { original: { modality: string; supported: boolean } }) =>
        a.original.modality === 'self_report' && !a.original.supported,
    ),
  ).toBe(true)
  expect(session.status).toBe('completed')
})

test('explanation and original support survive reload and ratings stay available', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Show & explain', exact: false }).click()
  await expect(page.locator('.practice-card .explanation')).toBeVisible()
  await expect(page.getByTestId('practice-phrase')).not.toContainText('_____')
  await page.reload()
  await expect(page.locator('.practice-card .explanation')).toBeVisible()
  await page.getByRole('button', { name: 'Recalled' }).click()
  const sessionId = page.url().split('/').pop()
  await expect
    .poll(async () => (await (await request.get(`/api/v1/sessions/${sessionId}`)).json()).cursor)
    .toBe(1)
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  expect(session.attempts[0].original.supported).toBe(true)
  expect(session.attempts[0].eligibility_reason).toBe('supported_learning_check')
})

test('lost rating acknowledgement is retried with the same event after reload', async ({
  page,
  request,
}) => {
  let lostId = ''
  let retryId = ''
  let lost = false
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.route('**/api/v1/sessions/*/events', async (route) => {
    if (route.request().method() !== 'POST') return route.continue()
    const event = route.request().postDataJSON().events[0]
    if (event.kind === 'rate' && !lost) {
      lost = true
      lostId = event.id
      await route.fetch()
      return route.fulfill({
        status: 503,
        contentType: 'application/json',
        body: '{"detail":"Acknowledgement lost"}',
      })
    }
    if (event.kind === 'rate') retryId = event.id
    await route.continue()
  })
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.getByRole('alert').first()).toContainText('Acknowledgement lost')
  await page.reload()
  await expect.poll(() => retryId).toBe(lostId)
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const sessionId = page.url().split('/').pop()
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  expect(session.cursor).toBe(1)
  expect(session.attempts.filter((a: { rating: string }) => a.rating)).toHaveLength(1)
})

test('language navigation preserves the pinned session and explicit resume', async ({ page }) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await page.getByLabel('Learning language', { exact: true }).selectOption('sr')
  await expect(page).toHaveURL(/\/learn\/sr$/)
  await expect(page.getByRole('button', { name: 'Nastavi vežbu' })).toBeVisible()
  await page.getByRole('button', { name: 'Nastavi vežbu' }).click()
  await expect(page).toHaveURL(/\/learn\/en\/practice\//)
  await expect(page.getByTestId('practice-phrase')).not.toContainText('_____')
  await page.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
  await page.getByRole('button', { name: 'Resume session', exact: true }).last().click()
  await expect(page.locator('.practice-card')).toBeVisible()
})

test('prepared dialogue supports an isolated turn and keeps the full exchange', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: /Catching up/ }).click()
  await expect(page.getByRole('heading', { name: 'Catching up' })).toBeVisible()
  await page.getByRole('button', { name: 'Rehearse dialogue', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await expect(page.locator('.dialogue-lines')).toContainText('It’s been a while')
  await page.getByRole('button', { name: 'Isolate this turn' }).click()
  await expect(page.locator('.dialogue-lines')).toHaveCount(0)
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.getByRole('heading', { name: 'Session complete' })).toBeVisible()
  const sessionId = page.url().split('/').pop()
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  expect(session.attempts[0].original.supported).toBe(true)
  expect(session.manifest.dialogue.content.turns).toHaveLength(2)
})

test('lookup between the cue and response remains support when returning to the drill', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const sessionId = page.url().split('/').pop()
  const initial = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  await page.goto(`/learn/en/phrases/${initial.manifest.targets[0].phrase.id}`)
  await expect(page.locator('.explanation')).toBeVisible()
  await page.getByRole('button', { name: 'Resume session', exact: true }).click()
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect
    .poll(async () => (await (await request.get(`/api/v1/sessions/${sessionId}`)).json()).cursor)
    .toBe(1)
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  expect(session.attempts[0].original.supported).toBe(true)
})

test('starting another language requires explicit takeover and preserves the old history', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const previousId = page.url().split('/').pop()
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await page.getByLabel('Learning language', { exact: true }).selectOption('sr')
  await page.getByRole('button', { name: 'Započni vežbu', exact: true }).click()
  expect((await (await request.get(`/api/v1/sessions/${previousId}`)).json()).status).toBe('active')
  await page
    .getByRole('button', { name: 'Započni izabranu vežbu umesto trenutne', exact: true })
    .click()
  await expect(page).toHaveURL(/\/learn\/sr\/practice\//)
  await expect(page.locator('.practice-card')).toBeVisible()
  const previous = await (await request.get(`/api/v1/sessions/${previousId}`)).json()
  expect(previous.status).toBe('superseded')
  expect(previous.attempts[0].original.supported).toBe(false)
})
