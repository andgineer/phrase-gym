import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession, retainedKeys } from './offline-actions'
import { retainedAudio, syntheticCapture } from './synthetic-capture'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})
test('offline capture retains every take, reopens, and transfers after its event receipts', async ({
  page,
  connection,
  request,
}) => {
  await syntheticCapture(page)
  const id = await downloadSession(page)
  const initial = await (await request.get(`/api/v1/sessions/${id}`)).json()
  const target = initial.manifest.targets[0].phrase.id
  connection.disconnect()
  for (let index = 0; index < 2; index++) {
    await page.getByRole('button', { name: 'Record', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Stop recording', exact: true })).toBeVisible()
    await expect.poll(() => retainedAudio(page)).toBe(index + 1)
    await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    if (await page.getByRole('button', { name: 'Stop playback', exact: true }).count())
      await page.getByRole('button', { name: 'Stop playback', exact: true }).click()
  }
  const takeIds = await retainedKeys(page, 'take:')
  await page.reload()
  await expect(page.getByRole('button', { name: 'My recording', exact: false })).toBeEnabled()
  connection.reconnect()
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect
    .poll(
      async () => (await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()).length,
    )
    .toBe(2)
  await expect.poll(() => retainedAudio(page)).toBe(0)
  const takes = await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()
  expect(takes.map((t: { id: string }) => `take:${t.id}`).sort()).toEqual(takeIds.sort())
  expect(takes.every((t: { complete: boolean }) => t.complete)).toBe(true)
})

test('an unanswered permission request can be cancelled and a late stream is released', async ({
  page,
}) => {
  await syntheticCapture(page, true)
  await downloadSession(page)
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect
    .poll(() =>
      page.evaluate(() =>
        Boolean((window as unknown as { __allowCapture?: () => void }).__allowCapture),
      ),
    )
    .toBe(true)
  await page.getByRole('button', { name: 'Cancel microphone request', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.evaluate(() => (window as unknown as { __allowCapture: () => void }).__allowCapture())
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as { __captureTrackState: string }).__captureTrackState,
      ),
    )
    .toBe('ended')
  await expect(page.getByRole('button', { name: 'Stop recording', exact: true })).toHaveCount(0)
})

test('recording quota failure keeps volatile audio visible and retry acknowledges the original take', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  const id = await downloadSession(page)
  const initial = await (await request.get(`/api/v1/sessions/${id}`)).json()
  const target = initial.manifest.targets[0].phrase.id
  await page.evaluate(() => {
    const put = IDBObjectStore.prototype.put
    IDBObjectStore.prototype.put = function (value, key) {
      if (String(key).startsWith('take:') && value.audio?.byteLength)
        throw new DOMException('Full', 'QuotaExceededError')
      return put.call(this, value, key)
    }
    Object.assign(window, {
      __restoreTakes: () => {
        IDBObjectStore.prototype.put = put
      },
    })
  })
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Stop recording', exact: true })).toBeVisible()
  await expect(page.getByRole('alert').filter({ hasText: 'Keep it open' })).toBeVisible()
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect(page.getByRole('button', { name: 'My recording', exact: false })).toBeEnabled()
  expect(await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()).toHaveLength(0)
  await page.evaluate(() => (window as unknown as { __restoreTakes: () => void }).__restoreTakes())
  await page.getByRole('button', { name: 'Retry', exact: true }).last().click()
  await expect
    .poll(
      async () => (await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()).length,
    )
    .toBe(1)
})
