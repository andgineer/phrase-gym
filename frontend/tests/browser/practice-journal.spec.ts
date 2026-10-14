import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('answer and advancement do not read unrelated retained buffers', async ({ page }) => {
  await page.addInitScript(() => {
    const open = IDBObjectStore.prototype.openCursor
    const visited: string[] = []
    Object.assign(window, { __journalVisited: visited })
    IDBObjectStore.prototype.openCursor = function (...args) {
      const request = open.apply(this, args)
      request.addEventListener('success', () => {
        if (request.result) visited.push(String(request.result.key))
      })
      return request
    }
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.evaluate(async () => {
    await new Promise<void>((resolve, reject) => {
      const opening = indexedDB.open('phrase-gym.practice.v1', 1)
      opening.onsuccess = () => {
        const transaction = opening.result.transaction('journal', 'readwrite')
        transaction.objectStore('journal').put(new ArrayBuffer(4 * 1024 * 1024), 'unrelated-audio')
        transaction.oncomplete = () => resolve()
        transaction.onabort = () => reject(transaction.error)
      }
    })
    ;(window as unknown as { __journalVisited: string[] }).__journalVisited.length = 0
  })
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
  const visited = await page.evaluate(
    () => (window as unknown as { __journalVisited: string[] }).__journalVisited,
  )
  expect(visited.some((key) => key.startsWith('event:'))).toBe(true)
  expect(visited).not.toContain('unrelated-audio')
})

test('a local checkpoint failure reports storage failure and retries its acknowledged event', async ({
  page,
}) => {
  await page.addInitScript(() => {
    const put = IDBObjectStore.prototype.put
    Object.assign(window, { __abortCheckpoint: false })
    IDBObjectStore.prototype.put = function (value, key) {
      const request = put.call(this, value, key)
      const state = window as unknown as { __abortCheckpoint: boolean }
      if (state.__abortCheckpoint && String(key).startsWith('session:')) {
        state.__abortCheckpoint = false
        queueMicrotask(() => this.transaction.abort())
      }
      return request
    }
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.evaluate(() => {
    ;(window as unknown as { __abortCheckpoint: boolean }).__abortCheckpoint = true
  })
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await expect(page.locator('.practice-panel').getByRole('alert')).toContainText(
    'Checkpoint write failed',
  )
  await expect(page.getByTestId('connection-required')).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeDisabled()
  await page.getByRole('button', { name: 'Retry', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toHaveCount(0)
  await expect(page.getByRole('alert')).toHaveCount(0)
})
