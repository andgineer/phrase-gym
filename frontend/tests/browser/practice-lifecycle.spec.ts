import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('a pending rating prevents old-card actions from reaching the next cue', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const id = page.url().split('/').pop()
  let release!: () => void
  const gate = new Promise<void>((resolve) => {
    release = resolve
  })
  let held = false
  await page.route('**/api/v1/sessions/*/events', async (route) => {
    if (route.request().postDataJSON().events[0].kind === 'rate') {
      const response = await route.fetch()
      held = true
      await gate
      return route.fulfill({ response })
    }
    await route.continue()
  })
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect.poll(() => held).toBe(true)
  try {
    for (const name of ['Record', 'Show answer', 'Show & explain', 'Reference']) {
      await expect(
        page.getByRole('button', { name, exact: name === 'Record' || name === 'Show answer' }),
      ).toBeDisabled()
    }
  } finally {
    release()
  }
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const session = await (await request.get(`/api/v1/sessions/${id}`)).json()
  expect(session.cursor).toBe(1)
  expect(session.attempts[1].revealed).toBe(false)
})

test('history selects the bookmarked session and keeps the open session resumable', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const previousUrl = page.url()
  const previousId = previousUrl.split('/').pop()
  await page.getByRole('button', { name: 'Show & explain' }).click()
  await expect(page.locator('.practice-card .explanation')).toBeVisible()
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page).not.toHaveURL(previousUrl)
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const currentUrl = page.url()
  const currentId = currentUrl.split('/').pop()
  await page.goBack()
  await page.goBack()
  await expect(page).toHaveURL(previousUrl)
  await expect(page.locator('.practice-card')).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toBeVisible()
  expect((await (await request.get(`/api/v1/sessions/${previousId}`)).json()).status).toBe(
    'superseded',
  )
  await page.getByRole('button', { name: 'Resume session', exact: true }).click()
  await expect(page).toHaveURL(currentUrl)
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  expect(
    (await (await request.get(`/api/v1/sessions/${currentId}`)).json()).attempts[0].revealed,
  ).toBe(false)
})

test('a competing review is reconciled without a permanently blocked journal', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const id = page.url().split('/').pop()
  const session = await (await request.get(`/api/v1/sessions/${id}`)).json()
  const response = await request.post(`/api/v1/sessions/${id}/events`, {
    data: {
      events: [
        {
          id: crypto.randomUUID(),
          sequence: session.last_sequence + 1,
          step: session.cursor,
          occurred_at: new Date().toISOString(),
          kind: 'rate',
          rating: 'recalled',
          modality: 'self_report',
        },
      ],
    },
  })
  expect(response.ok()).toBe(true)
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect
    .poll(async () => (await (await request.get(`/api/v1/sessions/${id}`)).json()).cursor)
    .toBe(2)
  await page.reload()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.getByRole('heading', { name: 'Session complete' })).toBeVisible()
})

test('a late reference acknowledgement cannot start playback after leaving practice', async ({
  page,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  let release!: () => void
  const gate = new Promise<void>((resolve) => {
    release = resolve
  })
  let held = false
  let acknowledged = false
  let eventId = ''
  await page.route('**/api/v1/sessions/*/events', async (route) => {
    if (route.request().postDataJSON().events[0].kind !== 'reference') return route.continue()
    eventId = route.request().postDataJSON().events[0].id
    const response = await route.fetch()
    held = true
    await gate
    await route.fulfill({ response })
    acknowledged = true
  })
  await page.getByRole('button', { name: 'Reference' }).click()
  await expect.poll(() => held).toBe(true)
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await expect(page.locator('.practice-card')).toHaveCount(0)
  release()
  await expect.poll(() => acknowledged).toBe(true)
  await expect
    .poll(() =>
      page.evaluate(
        async (id) =>
          new Promise<boolean>((resolve) => {
            const opening = indexedDB.open('phrase-gym.practice.v1', 1)
            opening.onsuccess = () => {
              const request = opening.result
                .transaction('journal')
                .objectStore('journal')
                .get(`event:${id}`)
              request.onsuccess = () => resolve(request.result === undefined)
            }
          }),
        eventId,
      ),
    )
    .toBe(true)
  await expect
    .poll(async () =>
      page.evaluate(
        () =>
          new Promise<number>((resolve) =>
            requestAnimationFrame(() =>
              requestAnimationFrame(() =>
                resolve(
                  (window as unknown as { __testPlaybackVolumes: number[] }).__testPlaybackVolumes
                    .length,
                ),
              ),
            ),
          ),
      ),
    )
    .toBe(0)
})
