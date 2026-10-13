import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession, retainedKeys } from './offline-actions'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})
test('a late downloaded outbox after another-device takeover remains retained without duplicate scheduling', async ({
  page,
  connection,
  request,
}) => {
  const id = await downloadSession(page)
  connection.disconnect()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.locator('.session-toolbar')).toContainText('2 / 3')
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.locator('.session-toolbar')).toContainText('3 / 3')
  const replacement = await request.post('/api/v1/sessions', {
    data: { id: crypto.randomUUID(), language: 'en', takeover_session_id: id },
  })
  expect(replacement.ok()).toBe(true)
  connection.reconnect()
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect.poll(() => retainedKeys(page, 'rejected-event:')).not.toHaveLength(0)
  await expect(page.getByRole('heading', { name: 'Results', exact: true }).first()).toBeVisible()
  const old = await (await request.get(`/api/v1/sessions/${id}`)).json()
  expect(old.status).toBe('superseded')
  expect(
    old.attempts.filter((a: { eligibility_reason: string }) => a.eligibility_reason === 'eligible'),
  ).toHaveLength(0)
  expect((await (await request.get('/api/v1/sessions/current')).json()).id).toBe(
    (await replacement.json()).id,
  )
})
test('an available update waits for a pause checkpoint and preserves the downloaded session', async ({
  page,
  connection,
}) => {
  await downloadSession(page)
  await page.evaluate(async () => {
    await navigator.serviceWorker.register('/sw.js?test-update=1')
  })
  await expect
    .poll(() =>
      page.evaluate(async () =>
        Boolean((await navigator.serviceWorker.getRegistration())?.waiting),
      ),
    )
    .toBe(true)
  await expect(page.getByRole('button', { name: 'Update app', exact: true })).toHaveCount(0)
  await page.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Update app', exact: true })).toBeVisible()
  await Promise.all([
    page.waitForEvent('load'),
    page.getByRole('button', { name: 'Update app', exact: true }).click(),
  ])
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
  connection.disconnect()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
})

test('downloaded rating with a lost committed receipt retries the original event once', async ({
  page,
  request,
}) => {
  const id = await downloadSession(page)
  await request.post('/_test/lose-rating')
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.locator('.session-toolbar')).toContainText('2 / 3')
  expect(await retainedKeys(page, 'event:')).not.toHaveLength(0)
  const queued = await retainedKeys(page, 'event:')
  const receipts = await (await request.get(`/api/v1/sessions/${id}/events`)).json()
  expect(receipts.filter((r: { id: string }) => queued.includes(`event:${r.id}`))).toHaveLength(1)
  await request.post('/_test/acknowledge-rating')
  await page.getByRole('button', { name: 'Retry', exact: true }).first().click()
  await expect.poll(() => retainedKeys(page, 'event:')).toHaveLength(0)
  const state = await (await request.get(`/api/v1/sessions/${id}`)).json()
  expect(state.cursor).toBe(1)
  expect(state.attempts.filter((a: { rating: string }) => a.rating)).toHaveLength(1)
})

test('changed publications keep offline text pinned and receive historical results without schedule changes', async ({
  page,
  connection,
  request,
}) => {
  const id = await downloadSession(page)
  const before = await page.getByTestId('practice-phrase').textContent()
  connection.disconnect()
  const update = await request.post('/_test/revise-content')
  expect(update.ok()).toBe(true)
  expect(await page.getByTestId('practice-phrase').textContent()).toBe(before)
  for (let cursor = 1; cursor <= 3; cursor++) {
    await page.getByRole('button', { name: 'Recalled' }).click()
    if (cursor < 3)
      await expect(page.locator('.session-toolbar')).toContainText(`${cursor + 1} / 3`)
  }
  await expect(
    page.getByRole('heading', { name: 'Finished on this device — awaiting confirmation' }),
  ).toBeVisible()
  connection.reconnect()
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect(page.getByRole('heading', { name: 'Session complete' })).toBeVisible()
  const state = await (await request.get(`/api/v1/sessions/${id}`)).json()
  expect(
    state.attempts.every(
      (a: { eligibility_reason: string }) => a.eligibility_reason === 'stale_content',
    ),
  ).toBe(true)
})
