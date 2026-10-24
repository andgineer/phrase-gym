import { randomUUID } from 'node:crypto'
import { expect } from '@playwright/test'
import { test } from './preview-server'
import { fillLesson } from './lesson-intake-helpers'
import { stageEnglishClaim } from './english-lesson'

test('the + panel hides the last batch items during a session and shows them outside it', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Add phrase', exact: true }).click()
  await fillLesson(page)
  await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
  const claim = await (
    await request.post('/api/v1/preparation/claims', { data: { claim_id: randomUUID() } })
  ).json()
  await stageEnglishClaim(request, claim)
  expect((await request.post('/api/v1/preparation/publish', { data: claim.fence })).ok()).toBe(true)
  const panel = page.locator('.phrase-drop-panel')
  await page.getByRole('button', { name: 'Check lesson status', exact: true }).click()
  await expect(panel.getByRole('link', { name: /Open published lesson/ })).toBeVisible()
  await expect(
    panel.getByRole('heading', { name: "Let's get started.", exact: true }),
  ).toBeVisible()
  await page.getByRole('button', { name: 'Close', exact: true }).click()
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toBeVisible()
  await page.getByRole('button', { name: 'Add phrase', exact: true }).click()
  await expect(panel.getByRole('link', { name: /Open published lesson/ })).toBeVisible()
  await expect(panel.getByRole('status')).toContainText('Published')
  await expect(panel).not.toContainText("Let's get started.")
  await page.getByRole('button', { name: 'Close', exact: true }).click()
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  await page.getByRole('button', { name: 'Add phrase', exact: true }).click()
  await expect(
    panel.getByRole('heading', { name: "Let's get started.", exact: true }),
  ).toBeVisible()
})
