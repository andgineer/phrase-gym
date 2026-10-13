import { expect } from '@playwright/test'
import type { Page } from '@playwright/test'
import type { PracticeSession } from '../../src/practiceTypes'
export async function startSession(page: Page) {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
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
export async function readCheckpoint(page: Page, id: string) {
  return page.evaluate(
    (id) =>
      new Promise<PracticeSession>((resolve, reject) => {
        const opening = indexedDB.open('phrase-gym.practice.v1', 1)
        opening.onsuccess = () => {
          const reading = opening.result
            .transaction('journal')
            .objectStore('journal')
            .get(`session:${id}`)
          reading.onsuccess = () => resolve(reading.result)
          reading.onerror = () => reject(reading.error)
        }
        opening.onerror = () => reject(opening.error)
      }),
    id,
  )
}
