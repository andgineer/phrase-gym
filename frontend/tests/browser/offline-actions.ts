import { expect } from '@playwright/test'
import type { Page } from '@playwright/test'
export async function downloadSession(page: Page) {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.evaluate(async () => {
    await navigator.serviceWorker.ready
  })
  await expect
    .poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller)))
    .toBe(true)
  await page.getByRole('button', { name: 'Download session', exact: true }).click()
  await expect(
    page.getByRole('status').filter({ hasText: 'Verified for offline practice' }),
  ).toBeVisible()
  return page.url().split('/').pop()!
}
export async function retainedKeys(page: Page, prefix: string): Promise<string[]> {
  return page.evaluate(
    (prefix) =>
      new Promise<string[]>((resolve, reject) => {
        const opening = indexedDB.open('phrase-gym.practice.v1', 1)
        opening.onsuccess = () => {
          const transaction = opening.result.transaction('journal')
          const keys = transaction.objectStore('journal').getAllKeys()
          keys.onsuccess = () =>
            resolve(keys.result.map(String).filter((key) => key.startsWith(prefix)))
          keys.onerror = () => reject(keys.error)
        }
      }),
    prefix,
  )
}
