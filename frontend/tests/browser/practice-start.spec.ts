import { expect } from '@playwright/test'
import { test } from './fixtures'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

for (const paused of [false, true]) {
  test(`Start session replaces an ${paused ? 'paused' : 'active'} session with the selected lesson in one click`, async ({
    page,
    request,
  }) => {
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Start session', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
    await expect(page.locator('.session-banner')).toHaveCount(0)
    const previousUrl = page.url()
    const previousId = previousUrl.split('/').pop()
    await page.getByRole('button', { name: 'Show answer', exact: true }).click()
    await expect
      .poll(
        async () =>
          (await (await request.get(`/api/v1/sessions/${previousId}`)).json()).attempts[0].revealed,
      )
      .toBe(true)
    if (paused) {
      await page.getByRole('button', { name: 'Pause', exact: true }).click()
      await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
      await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toHaveCount(1)
    }
    const previous = await (await request.get(`/api/v1/sessions/${previousId}`)).json()
    await page.getByRole('button', { name: 'Library', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toBeVisible()
    const lessons = await (await request.get('/api/v1/lessons?language=en&q=Making%20plans')).json()
    await page.getByLabel('Lesson to drill', { exact: true }).selectOption(lessons.items[0].id)
    await page.getByRole('button', { name: 'Start session', exact: true }).click()
    await expect(page).toHaveURL(/\/learn\/en\/practice\//)
    await expect(page).not.toHaveURL(previousUrl)
    await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
    await expect(page.locator('.session-banner')).toHaveCount(0)
    const current = await (await request.get('/api/v1/sessions/current')).json()
    expect(current.manifest.lesson_id).toBe(lessons.items[0].id)
    expect(current.cursor).toBe(0)
    const retained = await (await request.get(`/api/v1/sessions/${previousId}`)).json()
    expect(retained.status).toBe('superseded')
    expect(retained.manifest).toEqual(previous.manifest)
    expect(retained.attempts).toEqual(previous.attempts)
  })
}

test('Resume session keeps the existing session and cue instead of starting again', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
  const original = await (await request.get('/api/v1/sessions/current')).json()
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await page.getByRole('button', { name: 'Resume session', exact: true }).click()
  await expect(page).toHaveURL(`/learn/en/practice/${original.id}`)
  await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
  const resumed = await (await request.get('/api/v1/sessions/current')).json()
  expect(resumed.cursor).toBe(original.cursor)
  expect(resumed.manifest).toEqual(original.manifest)
})

test('a lost start acknowledgement retries the same new session and retains previous history', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const previousUrl = page.url()
  const previousId = previousUrl.split('/').pop()
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  const ids: string[] = []
  await page.route('**/api/v1/sessions', async (route) => {
    if (route.request().method() !== 'POST') return route.continue()
    ids.push(route.request().postDataJSON().id)
    if (ids.length !== 1) return route.continue()
    await route.fetch()
    await route.fulfill({ status: 503, json: { detail: 'Start acknowledgement lost' } })
  })
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('Start acknowledgement lost')
  await page.locator('.session-banner').getByRole('button', { name: 'Retry', exact: true }).click()
  await expect(page).toHaveURL(`/learn/en/practice/${ids[0]}`)
  expect(ids).toHaveLength(2)
  expect(ids[1]).toBe(ids[0])
  expect((await (await request.get(`/api/v1/sessions/${previousId}`)).json()).status).toBe(
    'superseded',
  )
  await expect(page.locator('.session-banner')).toHaveCount(0)
})

test('Start session waits for an outstanding answer receipt before replacing the session', async ({
  page,
  request,
}) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
  const previousId = page.url().split('/').pop()
  let release!: () => void
  const gate = new Promise<void>((resolve) => {
    release = resolve
  })
  let held = false
  let starts = 0
  await page.route('**/api/v1/sessions/*/events', async (route) => {
    if (route.request().postDataJSON().events[0].kind !== 'answer') return route.continue()
    const response = await route.fetch()
    held = true
    await gate
    await route.fulfill({ response })
  })
  await page.route('**/api/v1/sessions', async (route) => {
    if (route.request().method() === 'POST') starts += 1
    await route.continue()
  })
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await expect.poll(() => held).toBe(true)
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  try {
    await expect(page.locator('.session-banner').getByRole('status')).toHaveText('Saving…')
    expect(starts).toBe(0)
  } finally {
    release()
  }
  await expect(page).toHaveURL(/\/learn\/en\/practice\//)
  await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
  const retained = await (await request.get(`/api/v1/sessions/${previousId}`)).json()
  expect(retained.status).toBe('superseded')
  expect(retained.attempts[0].revealed).toBe(true)
  expect(retained.attempts[0].original.modality).toBe('self_report')
  expect(starts).toBe(1)
})
