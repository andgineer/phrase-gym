import { randomUUID } from 'node:crypto'
import { expect } from '@playwright/test'
import { test } from './preview-server'
import { attachAudio, fillLesson } from './lesson-intake-helpers'

for (const terminal of ['exhausted', 'cancelled']) {
  test(`accepted ${terminal} preparation can be left locally without changing Server work`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
    const receiptResponse = page.waitForResponse(
      (r) => r.url().endsWith('/api/v1/inbox/lessons') && r.request().method() === 'POST',
    )
    await fillLesson(page)
    const receipt = await (await receiptResponse).json()
    const statusPath = `/api/v1/inbox/lessons/${receipt.generation_id}`
    await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
    if (terminal === 'cancelled') {
      expect((await request.post(`${statusPath}/cancel`)).ok()).toBe(true)
    } else {
      for (let attempt = 0; attempt < 3; attempt++) {
        const claim = await (
          await request.post('/api/v1/preparation/claims', { data: { claim_id: randomUUID() } })
        ).json()
        expect(claim.fence.generation_id).toBe(receipt.generation_id)
        expect(
          (
            await request.post('/api/v1/preparation/fail', {
              data: { fence: claim.fence, reason: 'Synthetic exhausted preparation' },
            })
          ).ok(),
        ).toBe(true)
        if (attempt < 2) expect((await request.post(`${statusPath}/retry`)).ok()).toBe(true)
      }
    }
    await page.getByRole('button', { name: 'Check lesson status', exact: true }).click()
    await expect(
      page.getByText(
        terminal === 'cancelled'
          ? /Preparation was cancelled/
          : /Preparation failed. Check the reason/,
      ),
    ).toBeVisible()
    if (terminal === 'exhausted') {
      await page.getByRole('button', { name: 'Retry preparation', exact: true }).click()
      await expect(page.getByRole('alert').filter({ hasText: 'attempt limit' })).toBeVisible()
    }
    const before = await (await request.get(statusPath)).json()
    await page.getByRole('button', { name: /Discard local saved input/ }).click()
    await expect(page.getByLabel('Lesson title', { exact: true })).toBeVisible()
    await page.reload()
    await page.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
    await fillLesson(page, 'Replacement lesson')
    await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
    await expect(
      page.getByRole('heading', { name: 'Replacement lesson', exact: true }),
    ).toBeVisible()
    expect(await (await request.get(statusPath)).json()).toEqual(before)
    if (terminal === 'exhausted')
      expect((await request.post(`${statusPath}/retry`)).status()).toBe(409)
  })
}

test('another open intake preserves a response-lost delivery through reload and receipt reconciliation', async ({
  page,
  context,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  const other = await context.newPage()
  for (const tab of [page, other]) {
    await tab.goto('/learn/en')
    await tab.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
    await fillLesson(tab, 'Opening a meeting', false)
  }
  await other.getByLabel('Lesson title', { exact: true }).fill('Unsent second lesson')
  const deliveries: unknown[] = []
  let lost = true
  await context.route('**/api/v1/inbox/lessons', async (route) => {
    deliveries.push(route.request().postDataJSON())
    const response = await route.fetch()
    expect(response.ok(), await response.text()).toBe(true)
    if (lost) {
      lost = false
      await route.abort()
    } else await route.fulfill({ response })
  })
  await attachAudio(page)
  await expect(page.getByTestId('connection-required')).toBeVisible()
  await attachAudio(other)
  await expect(other.getByRole('alert')).toContainText('changed in another tab')
  await expect(other.getByRole('heading', { name: 'Opening a meeting', exact: true })).toBeVisible()
  expect(deliveries).toHaveLength(1)
  await page.close()
  await other.reload()
  await other.getByRole('button', { name: 'Add a first lesson', exact: true }).click()
  await expect(other.getByText(/Delivery is unconfirmed/)).toBeVisible()
  await other.getByRole('button', { name: 'Retry the same delivery', exact: true }).click()
  await expect(other.getByText(/Accepted — waiting for Coach/)).toBeVisible()
  expect(deliveries).toHaveLength(1)
})
