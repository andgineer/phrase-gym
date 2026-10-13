import { expect } from '@playwright/test'
import { test } from './fixtures'

test('a delayed session read cannot overwrite an acknowledged evidence checkpoint', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const id = page.url().split('/').pop()!
  let releaseWrite!: () => void
  let releaseRead!: () => void
  const writeGate = new Promise<void>((resolve) => {
    releaseWrite = resolve
  })
  const readGate = new Promise<void>((resolve) => {
    releaseRead = resolve
  })
  let reading = false
  let writing = false
  await page.route('**/api/v1/sessions/*/events', async (route) => {
    if (route.request().postDataJSON().events[0].kind !== 'answer') return route.continue()
    writing = true
    await writeGate
    await route.continue()
  })
  await page.route('**/api/v1/sessions/current', async (route) => {
    const response = await route.fetch()
    reading = true
    await readGate
    await route.fulfill({ response })
  })
  async function checkpoint() {
    return page.evaluate(
      (id) =>
        new Promise<{ last_sequence: number; attempts: { revealed: boolean }[] }>((resolve) => {
          const opening = indexedDB.open('phrase-gym.practice.v1', 1)
          opening.onsuccess = () => {
            const read = opening.result
              .transaction('journal')
              .objectStore('journal')
              .get(`session:${id}`)
            read.onsuccess = () => resolve(read.result)
          }
        }),
      id,
    )
  }
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await expect.poll(() => writing).toBe(true)
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await expect.poll(() => reading).toBe(true)
  try {
    releaseWrite()
    await expect.poll(async () => (await checkpoint()).last_sequence).toBe(2)
    releaseRead()
    await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toBeEnabled()
    const retained = await checkpoint()
    expect(retained.last_sequence).toBe(2)
    expect(retained.attempts[0]!.revealed).toBe(true)
  } finally {
    releaseWrite()
    releaseRead()
  }
})
