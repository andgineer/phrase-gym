import { expect } from '@playwright/test'
import { test } from './connection-fixtures'
import { startSession, retainedKeys } from './session-actions'
import { syntheticCapture, retainedAudio } from './synthetic-capture'

test('retires the previous worker and packs while preserving pending audio and original events', async ({
  page,
  request,
  connection,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await syntheticCapture(page)
  const id = await startSession(page)
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  connection.disconnect()
  await page.evaluate(() => window.dispatchEvent(new Event('offline')))
  await page.getByRole('button', { name: 'Stop or cancel audio', exact: true }).click()
  await expect.poll(() => retainedKeys(page, 'event:')).toHaveLength(1)
  const eventKeys = await retainedKeys(page, 'event:')
  const takeKeys = await retainedKeys(page, 'take-v2:')
  await request.post('/_test/pending-delivery/true')
  connection.reconnect()
  await request.post('/_test/legacy-worker/true')
  await page.evaluate(async () => {
    await navigator.serviceWorker.register('/sw.js')
    await navigator.serviceWorker.ready
    const media = await caches.open('phrase-gym.selected-media.v1')
    await media.put('/api/v1/media/obsolete', new Response('obsolete media'))
    const unrelated = await caches.open('unrelated-test-cache')
    await unrelated.put('/other', new Response('unrelated'))
    await new Promise<void>((resolve, reject) => {
      const opening = indexedDB.open('phrase-gym.practice.v1', 1)
      opening.onsuccess = () => {
        const transaction = opening.result.transaction('journal', 'readwrite')
        transaction.objectStore('journal').put({ version: 3 }, 'pack:obsolete')
        transaction.oncomplete = () => resolve()
        transaction.onerror = () => reject(transaction.error)
      }
    })
  })
  await expect
    .poll(() =>
      page.evaluate(async () => (await navigator.serviceWorker.getRegistrations()).length),
    )
    .toBe(1)
  expect(await page.evaluate(() => caches.keys())).toContain('phrase-gym.shell.legacy-test')
  await request.post('/_test/legacy-worker/false')
  await page.reload()
  await expect(page.getByTestId('connection-required')).toBeVisible()
  await expect
    .poll(() =>
      page.evaluate(async () => (await navigator.serviceWorker.getRegistrations()).length),
    )
    .toBe(0)
  expect(await page.evaluate(() => caches.keys())).toEqual(['unrelated-test-cache'])
  expect(await retainedKeys(page, 'pack:')).toHaveLength(0)
  expect(await retainedKeys(page, 'event:')).toEqual(eventKeys)
  expect(await retainedKeys(page, 'take-v2:')).toEqual(takeKeys)
  expect(await retainedKeys(page, 'session:')).toContain(`session:${id}`)
  await request.post('/_test/pending-delivery/false')
  await page
    .getByTestId('connection-required')
    .getByRole('button', { name: 'Retry', exact: true })
    .click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await expect.poll(() => retainedAudio(page)).toBe(0)
  const state = await (await request.get(`/api/v1/sessions/${id}`)).json()
  const takes = await (
    await request.get(`/api/v1/phrases/${state.attempts[0].target_id}/recordings`)
  ).json()
  expect(takes).toHaveLength(1)
  expect(`take-v2:${takes[0].id}`).toBe(takeKeys[0])
  const receipts = await (await request.get(`/api/v1/sessions/${id}/events`)).json()
  expect(receipts.filter((r: { id: string }) => `event:${r.id}` === eventKeys[0])).toHaveLength(1)
})
