import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession, retainedKeys } from './offline-actions'

test('an existing pack loses readiness after shell eviction and recovers without deleting evidence', async ({
  page,
  request,
  connection,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await downloadSession(page)
  connection.disconnect()
  await page.getByRole('button', { name: 'Show & explain' }).click()
  await expect(page.locator('.explanation')).toBeVisible()
  const events = await retainedKeys(page, 'event:')
  expect(events.length).toBeGreaterThan(0)
  await page.evaluate(async () => {
    const name = (await caches.keys()).find((name) => name.startsWith('phrase-gym.shell.'))!
    const cache = await caches.open(name)
    const entry = (await cache.keys()).find((entry) => entry.url.endsWith('.js'))!
    await cache.delete(entry)
  })
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(
    page.getByRole('alert').filter({ hasText: 'app shell is unavailable' }).first(),
  ).toBeVisible()
  await expect(page.getByText('Verified for offline practice')).toHaveCount(0)
  expect(await retainedKeys(page, 'event:')).toEqual(events)
  await expect(page.locator('.session-toolbar')).toContainText('1 / 3')
  connection.reconnect()
  await page.reload()
  await expect(page.getByText('Verified for offline practice')).toHaveCount(0)
  await expect(
    page.getByRole('alert').filter({ hasText: 'download this session again' }).first(),
  ).toBeVisible()
  await expect.poll(() => retainedKeys(page, 'event:')).toEqual([])
  await page.getByRole('button', { name: 'Download session', exact: true }).click()
  await expect(page.getByText('Verified for offline practice')).toBeVisible()
  connection.disconnect()
  await page.reload()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await expect(page.locator('.explanation')).toBeVisible()
})
