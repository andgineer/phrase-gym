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

test('a foreground read blocks dialogue introduction and detour until restoration finishes', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  const dialogue = (await (await request.get('/api/v1/dialogues?language=en')).json()).items[0]
  const state = await (
    await request.post('/api/v1/sessions', {
      data: { id: crypto.randomUUID(), language: 'en', dialogue_id: dialogue.id },
    })
  ).json()
  await page.goto(`/learn/en/practice/${state.id}`)
  await expect(page.getByRole('button', { name: 'Rehearse dialogue', exact: true })).toBeEnabled()
  for (const begun of [false, true]) {
    if (begun) {
      await page.getByRole('button', { name: 'Rehearse dialogue', exact: true }).click()
      await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    }
    const before = await (await request.get(`/api/v1/sessions/${state.id}`)).json()
    let reading = false
    let release!: () => void
    const gate = new Promise<void>((resolve) => {
      release = resolve
    })
    await page.route('**/api/v1/sessions/current', async (route) => {
      const response = await route.fetch()
      reading = true
      await gate
      await route.fulfill({ response })
    })
    await page.evaluate(() => window.dispatchEvent(new Event('online')))
    await expect.poll(() => reading).toBe(true)
    try {
      const action = page.getByRole('button', {
        name: begun ? 'Isolate this turn' : 'Rehearse dialogue',
        exact: true,
      })
      await expect(action).toBeDisabled()
      await expect(
        page.getByRole('button', { name: 'Download session', exact: true }),
      ).toBeDisabled()
      await action.evaluate((button: HTMLButtonElement) => button.click())
      expect((await (await request.get(`/api/v1/sessions/${state.id}`)).json()).last_sequence).toBe(
        before.last_sequence,
      )
    } finally {
      release()
    }
    await expect(
      page.getByRole('button', {
        name: begun ? 'Isolate this turn' : 'Rehearse dialogue',
        exact: true,
      }),
    ).toBeEnabled()
    await page.unroute('**/api/v1/sessions/current')
  }
})
