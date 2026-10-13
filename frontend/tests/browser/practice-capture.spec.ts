import { expect } from '@playwright/test'
import { test } from './capture-fixtures'
import { retainedAudio, syntheticCapture } from './synthetic-capture'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('actual encoded takes are retained separately and the original response stays unchanged', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const sessionId = page.url().split('/').pop()
  let targetId = ''
  for (let i = 0; i < 2; i++) {
    await page.getByRole('button', { name: 'Record', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Stop recording', exact: true })).toBeVisible()
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeDisabled()
    await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
    await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
    await expect(page.getByText('Press again to stop recording', { exact: true })).toHaveCount(0)
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toHaveCount(0)
    const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
    targetId = session.attempts[0].target_id
    expect(session.attempts[0].original).toMatchObject({ modality: 'recorded', supported: false })
    await expect
      .poll(
        async () =>
          (await (await request.get(`/api/v1/phrases/${targetId}/recordings`)).json()).length,
      )
      .toBe(i + 1)
    if (await page.getByRole('button', { name: 'Stop playback', exact: true }).count())
      await page.getByRole('button', { name: 'Stop playback', exact: true }).click()
  }
  expect(
    await page.evaluate(
      () => (window as unknown as { __captureRequests: number }).__captureRequests,
    ),
  ).toBe(1)
  const retained = await (await request.get(`/api/v1/phrases/${targetId}/recordings`)).json()
  expect(
    retained.every(
      (take: { complete: boolean; byte_length: number }) => take.complete && take.byte_length > 0,
    ),
  ).toBe(true)
  await page.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
})

test('a pending take survives a lost acknowledgement and reload without duplication', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  let id = ''
  let retried = ''
  let lost = false
  await page.route('**/api/v1/recordings', async (route) => {
    const payload = route.request().postDataJSON()
    if (!lost) {
      lost = true
      id = payload.id
      await route.fetch()
      return route.fulfill({
        status: 503,
        contentType: 'application/json',
        body: '{"detail":"Take acknowledgement lost"}',
      })
    }
    retried = payload.id
    await route.continue()
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect(page.getByRole('alert').first()).toContainText('Take acknowledgement lost')
  await page.reload()
  await expect.poll(() => retried).toBe(id)
  await expect.poll(() => retainedAudio(page)).toBe(0)
  const sessionId = page.url().split('/').pop()
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  const rows = await (
    await request.get(`/api/v1/phrases/${session.attempts[0].target_id}/recordings`)
  ).json()
  expect(rows).toHaveLength(1)
})

test('recoverable interrupted capture is retained as incomplete after reload', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  const sessionId = page.url().split('/').pop()
  await page.reload()
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  const path = `/api/v1/phrases/${session.attempts[0].target_id}/recordings`
  await expect.poll(async () => (await (await request.get(path)).json()).length).toBe(1)
  const rows = await (await request.get(path)).json()
  expect(rows[0].complete).toBe(false)
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect
    .poll(async () => (await (await request.get(`/api/v1/sessions/${sessionId}`)).json()).cursor)
    .toBe(1)
  const rated = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  expect(rated.attempts[0].eligibility_reason).toBe('technical_failure')
})

test('lookup recording stays supported without creating a review session', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  const targetId = page.url().split('/').pop()?.split('?')[0]
  await expect
    .poll(
      async () =>
        (await (await request.get(`/api/v1/phrases/${targetId}/recordings`)).json()).length,
    )
    .toBe(1)
  expect(await (await request.get('/api/v1/sessions/current')).json()).toBeNull()
})

test('permission denial keeps self-check available without inventing a memory failure', async ({
  page,
  request,
}) => {
  await page.addInitScript(() =>
    Object.defineProperty(navigator, 'mediaDevices', {
      value: {
        getUserMedia: async () => {
          throw new DOMException('Permission denied', 'NotAllowedError')
        },
      },
    }),
  )
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const sessionId = page.url().split('/').pop()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect(page.getByRole('alert').first()).toContainText('Permission denied')
  await page.getByRole('button', { name: 'Not yet' }).click()
  await expect
    .poll(async () => (await (await request.get(`/api/v1/sessions/${sessionId}`)).json()).cursor)
    .toBe(1)
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  expect(session.attempts[0].eligibility_reason).toBe('technical_failure')
})

test('pending audio transfer cannot block the next self-check or keep the microphone after completion', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  let release!: () => void
  const gate = new Promise<void>((resolve) => {
    release = resolve
  })
  let pending = false
  await page.route('**/api/v1/recordings', async (route) => {
    pending = true
    await gate
    await route.continue()
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect.poll(() => pending).toBe(true)
  try {
    for (let index = 0; index < 3; index++) {
      await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
      await page.getByRole('button', { name: 'Recalled' }).click()
    }
    await expect(page.getByRole('heading', { name: 'Session complete' })).toBeVisible()
    await expect
      .poll(() =>
        page.evaluate(
          () => (window as unknown as { __captureTrackState: string }).__captureTrackState,
        ),
      )
      .toBe('ended')
  } finally {
    release()
  }
  const sessionId = page.url().split('/').pop()
  const session = await (await request.get(`/api/v1/sessions/${sessionId}`)).json()
  await expect
    .poll(
      async () =>
        (
          await (
            await request.get(`/api/v1/phrases/${session.attempts[0].target_id}/recordings`)
          ).json()
        ).length,
    )
    .toBe(1)
})
