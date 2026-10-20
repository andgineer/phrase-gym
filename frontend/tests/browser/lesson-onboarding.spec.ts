import { randomUUID } from 'node:crypto'
import { expect } from '@playwright/test'
import { test } from './preview-server'
import { fillLesson } from './lesson-intake-helpers'
import { stageEnglishClaim } from './english-lesson'

test('first lesson retains delivery identity and shows actual preparation and rehearsal readiness', async ({
  page,
  request,
  preview,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  const deliveries: unknown[] = []
  let lost = true
  await page.route('**/api/v1/inbox/lessons', async (route) => {
    deliveries.push(route.request().postDataJSON())
    const response = await route.fetch()
    expect(response.ok(), await response.text()).toBe(true)
    if (lost) {
      lost = false
      await route.abort()
    } else await route.fulfill({ response })
  })
  await page.goto('/learn/en')
  await expect(page.getByText(/No published phrases in this language yet/)).toBeVisible()
  await expect(page.getByRole('button', { name: 'Start session', exact: true })).toBeDisabled()
  await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
  await page.getByRole('button', { name: 'Close', exact: true }).click()
  expect(deliveries).toHaveLength(0)
  await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
  await fillLesson(page)
  await expect(page.getByTestId('connection-required')).toBeVisible()
  await page
    .getByTestId('connection-required')
    .getByRole('button', { name: 'Retry', exact: true })
    .click()
  await page.getByRole('button', { name: 'Retry the same delivery', exact: true }).click()
  await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
  expect(deliveries).toHaveLength(1)
  await preview.restart()
  await page.reload()
  await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
  await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
  const claim = await (
    await request.post('/api/v1/preparation/claims', { data: { claim_id: randomUUID() } })
  ).json()
  await page.getByRole('button', { name: 'Check lesson status', exact: true }).click()
  await expect(
    page.getByText('Coach is preparing the accepted lesson.', { exact: true }),
  ).toBeVisible()
  expect(
    (
      await request.post('/api/v1/preparation/fail', {
        data: {
          fence: claim.fence,
          reason: 'Synthetic preparation failed; correct the supplied lesson.',
        },
      })
    ).ok(),
  ).toBe(true)
  await page.getByRole('button', { name: 'Check lesson status', exact: true }).click()
  await expect(page.getByText(/Preparation failed. Check the reason/)).toBeVisible()
  await expect(page.getByRole('alert')).toContainText('Synthetic preparation failed')
  await page.getByRole('button', { name: 'Retry preparation', exact: true }).click()
  await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
  const retry = await (
    await request.post('/api/v1/preparation/claims', { data: { claim_id: randomUUID() } })
  ).json()
  await stageEnglishClaim(request, retry)
  await page.getByRole('button', { name: 'Check lesson status', exact: true }).click()
  await expect(
    page.getByText('Preparation finished; waiting for confirmed publication.', { exact: true }),
  ).toBeVisible()
  expect((await request.post('/api/v1/preparation/publish', { data: retry.fence })).ok()).toBe(true)
  await page.getByRole('button', { name: 'Check lesson status', exact: true }).click()
  await page.getByRole('link', { name: /Open published lesson/ }).click()
  await expect(page.getByText(/Ready for ungraded study/)).toBeVisible()
  await expect(page.getByRole('button', { name: 'Start lesson drill', exact: true })).toBeEnabled()
  await page.getByRole('link', { name: /Let's get started/ }).click()
  await expect(page.getByRole('heading', { name: "Let's get started.", exact: true })).toBeVisible()
  await page.getByRole('link', { name: 'Phrase Gym' }).click()
  await expect(page.getByText(/No published phrases in this language yet/)).toHaveCount(0)
  await expect(page.getByText(/Ready for ungraded study: 1/)).toBeVisible()
  await expect(page.getByRole('button', { name: 'Start session', exact: true })).toBeEnabled()
  await page.getByLabel('Learning language', { exact: true }).selectOption('sr')
  await page.getByRole('button', { name: 'Dodaj prvu lekciju', exact: true }).click()
  await expect(page.getByText(/Unos novih lekcija trenutno podržava samo engleski/)).toBeVisible()
  await expect(page.getByRole('button', { name: 'Pošalji lekciju', exact: true })).toHaveCount(0)
})

test('private-network denial retains unconfirmed input and explains unavailable intake', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.route('**/api/v1/media', (route) =>
    route.fulfill({
      status: 403,
      contentType: 'application/json',
      body: JSON.stringify({ detail: 'Use the private connection' }),
    }),
  )
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
  await fillLesson(page)
  await expect(page.getByRole('alert')).toContainText('This connection cannot submit material')
  await expect(page.getByText(/Delivery is unconfirmed/)).toBeVisible()
  await page.reload()
  await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Opening a meeting', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Retry the same delivery', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('This connection cannot submit material')
  expect(
    await (
      await request.post('/api/v1/preparation/claims', { data: { claim_id: randomUUID() } })
    ).json(),
  ).toBeNull()
})

test('unavailable local storage blocks sending before any lesson delivery', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.addInitScript(() => {
    const open = indexedDB.open.bind(indexedDB)
    indexedDB.open = function (name: string, version?: number) {
      if (name === 'phrase-gym.inbox.v1') throw new Error('Synthetic denied storage')
      return open(name, version)
    }
  })
  let deliveries = 0
  page.on('request', (r) => {
    if (r.url().endsWith('/api/v1/inbox/lessons') && r.method() === 'POST') deliveries++
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('Could not save input on this device')
  await expect(page.getByLabel(/Original reference audio/)).toBeDisabled()
  expect(deliveries).toBe(0)
})
