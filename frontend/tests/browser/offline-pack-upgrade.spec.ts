import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession, retainedKeys } from './offline-actions'

test('an older pack requires a fresh download and retains queued actions', async ({
  page,
  request,
  connection,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  const id = await downloadSession(page)
  connection.disconnect()
  await page.getByRole('button', { name: 'Show & explain' }).click()
  await expect(page.locator('.explanation')).toBeVisible()
  const pending = await retainedKeys(page, 'event:')
  expect(pending.length).toBeGreaterThan(0)
  await page.evaluate(
    (id) =>
      new Promise<void>((resolve, reject) => {
        const opening = indexedDB.open('phrase-gym.practice.v1', 1)
        opening.onsuccess = () => {
          const transaction = opening.result.transaction('journal', 'readwrite')
          const store = transaction.objectStore('journal')
          const key = `pack:${id}`
          const reading = store.get(key)
          reading.onsuccess = () => store.put({ ...reading.result, version: 1 }, key)
          transaction.oncomplete = () => resolve()
          transaction.onerror = () => reject(transaction.error)
        }
        opening.onerror = () => reject(opening.error)
      }),
    id,
  )
  await page.reload()
  await expect(
    page.getByRole('alert').filter({ hasText: 'fresh download while connected' }).first(),
  ).toBeVisible()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeDisabled()
  expect(await retainedKeys(page, 'event:')).toEqual(pending)
  expect(await retainedKeys(page, 'pack:')).toContain(`pack:${id}`)
  connection.reconnect()
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect.poll(() => retainedKeys(page, 'event:')).toEqual([])
  await page.getByRole('button', { name: 'Download session', exact: true }).click()
  await expect(page.getByText('Verified for offline practice')).toBeVisible()
  connection.disconnect()
  await page.reload()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await expect(page.locator('.explanation')).toBeVisible()
})
