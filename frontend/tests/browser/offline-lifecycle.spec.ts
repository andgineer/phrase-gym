import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession } from './offline-actions'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})
test('Pause as the first disconnected action persists after offline and empty-outbox online reloads', async ({
  page,
  connection,
  request,
}) => {
  const id = await downloadSession(page)
  connection.disconnect()
  await page.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toBeEnabled()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
  connection.reconnect()
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect(page.locator('.practice-panel')).toHaveAttribute('aria-busy', 'false')
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
  expect((await (await request.get(`/api/v1/sessions/${id}`)).json()).status).toBe('active')
  await page.getByRole('button', { name: 'Resume session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
})
test('Resume as the first disconnected action overrides a server-paused checkpoint after reopening', async ({
  page,
  connection,
  request,
}) => {
  const id = await downloadSession(page)
  await page.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toBeEnabled()
  expect((await (await request.get(`/api/v1/sessions/${id}`)).json()).status).toBe('paused')
  connection.disconnect()
  await page.getByRole('button', { name: 'Resume session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.reload()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  connection.reconnect()
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect(page.locator('.practice-panel')).toHaveAttribute('aria-busy', 'false')
  await page.reload()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
})
test('a real lifecycle version conflict is not disguised as an offline transition', async ({
  page,
  request,
}) => {
  const id = await downloadSession(page)
  const state = await (await request.get(`/api/v1/sessions/${id}`)).json()
  const response = await request.patch(`/api/v1/sessions/${id}`, {
    data: { expected_version: state.version, status: 'paused' },
  })
  expect(response.ok()).toBe(true)
  const paused = await response.json()
  expect(
    (
      await request.patch(`/api/v1/sessions/${id}`, {
        data: { expected_version: paused.version, status: 'active' },
      })
    ).ok(),
  ).toBe(true)
  await page.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect(page.getByRole('alert').first()).toContainText('version')
  await expect(page.getByRole('heading', { name: 'Session paused' })).toHaveCount(0)
})
