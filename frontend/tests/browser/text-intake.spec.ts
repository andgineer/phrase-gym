import { expect } from '@playwright/test'
import { test } from './preview-server'
import { prepareTextWithNativeFakes } from './text-intake'

test('ordinary text intake generates verified reference bytes and normal Start survives Server restart without Coach', async ({
  page,
  request,
  preview,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'en', ui_language_mode: 'learning' },
  })
  const { publication, reference } = await prepareTextWithNativeFakes(preview.url)
  expect(publication.target_ids).toHaveLength(1)
  const referenceResponse = await request.get(`/api/v1/media/${reference}`)
  expect(referenceResponse.ok()).toBe(true)
  const bytes = await referenceResponse.body()
  expect(bytes.subarray(0, 4).toString()).toBe('RIFF')
  await page.goto('/learn/en')
  await expect(page.getByText(/Ready for ungraded study: 1/)).toBeVisible()
  await page.getByLabel('Lesson to drill', { exact: true }).selectOption(publication.lesson_id)
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page).toHaveURL(/\/learn\/en\/practice\//)
  await expect(page.getByRole('heading', { name: "Let's get started.", exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: /Reference$/ })).toBeEnabled()
  const current = await (await request.get('/api/v1/sessions/current')).json()
  expect(current.manifest.lesson_id).toBe(publication.lesson_id)
  for (const restart of [false, true]) {
    if (restart) await preview.restart()
    await page.reload()
    await expect(
      page.getByRole('heading', { name: "Let's get started.", exact: true }),
    ).toBeVisible()
    await expect(page.getByRole('button', { name: /Reference$/ })).toBeEnabled()
    expect(await (await request.get(`/api/v1/media/${reference}`)).body()).toEqual(bytes)
    expect(await (await request.get('/api/v1/sessions/current')).json()).toEqual(current)
  }
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Resume session', exact: true }).click()
  await page.getByRole('button', { name: 'Next', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
  const studied = await (await request.get(`/api/v1/sessions/${current.id}`)).json()
  expect(studied.attempts[0].rating).toBeNull()
  const studyReceipts = await (await request.get(`/api/v1/sessions/${current.id}/events`)).json()
  expect(
    studyReceipts.every((receipt: { schedule_changed: boolean }) => !receipt.schedule_changed),
  ).toBe(true)
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toHaveCount(0)
  await page.reload()
  await expect(page.getByRole('button', { name: 'Resume session', exact: true })).toHaveCount(0)
  await expect(page.getByText(/New: 1/)).toBeVisible()
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
  await expect(page.locator('.practice-prompt')).toHaveText(
    'Everyone has arrived. Open the meeting.',
  )
  await expect(page.getByRole('button', { name: /Recalled$/ })).toBeDisabled()
  const recall = await (await request.get('/api/v1/sessions/current')).json()
  expect(recall.id).not.toBe(current.id)
  expect(recall.manifest.targets[0].phrase.id).toBe(publication.target_ids[0])
  expect(recall.manifest.targets[0].eligibility).toBe('new')
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toHaveText("Let's get started.")
  await page.getByRole('button', { name: /Recalled$/ }).click()
  await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
  const assessed = await (await request.get(`/api/v1/sessions/${recall.id}`)).json()
  expect(assessed.attempts[0].original.supported).toBe(false)
  expect(assessed.attempts[0].eligibility_reason).toBe('eligible')
  const recallReceipts = await (await request.get(`/api/v1/sessions/${recall.id}/events`)).json()
  expect(recallReceipts.at(-1)).toMatchObject({
    schedule_changed: true,
    schedule_version: 1,
    eligibility_reason: 'eligible',
  })
})

test('delayed current-session read cannot resurrect a completed cross-tab checkpoint', async ({
  page,
  context,
  request,
  preview,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'en', ui_language_mode: 'learning' },
  })
  await prepareTextWithNativeFakes(preview.url)
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Next', exact: true })).toBeEnabled()
  const other = await context.newPage()
  let release!: () => void
  let arrived!: () => void
  const delayed = new Promise<void>((resolve) => {
    arrived = resolve
  })
  await other.route('**/api/v1/sessions/current', async (route) => {
    const response = await route.fetch()
    arrived()
    await new Promise<void>((resolve) => {
      release = resolve
    })
    await route.fulfill({ response })
  })
  const loading = other.goto('/learn/en')
  await delayed
  await page.getByRole('button', { name: 'Next', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
  release()
  await loading
  await expect(other.getByRole('button', { name: 'Resume session', exact: true })).toHaveCount(0)
})
