import { createHash } from 'node:crypto'
import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession, retainedKeys } from './offline-actions'
import { syntheticCapture } from './synthetic-capture'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})
for (const navigation of ['library', 'settings', 'language']) {
  test(`quota-failed audio survives ${navigation} navigation and acknowledges its original bytes once`, async ({
    page,
    request,
  }) => {
    await syntheticCapture(page)
    const id = await downloadSession(page)
    const state = await (await request.get(`/api/v1/sessions/${id}`)).json()
    const target = state.manifest.targets[0].phrase.id
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
    await expect(page.getByRole('alert').filter({ hasText: 'Keep it open' })).toBeVisible()
    await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    const originalId = (await retainedKeys(page, 'take:'))[0]!.slice(5)
    if (navigation === 'library') {
      await page.getByRole('button', { name: 'Library', exact: true }).click()
      await expect(page.getByRole('alert').filter({ hasText: 'Keep it open' })).toBeVisible()
      await page.getByRole('button', { name: 'Resume session', exact: true }).click()
    } else if (navigation === 'settings') {
      await page.getByRole('button', { name: 'Settings', exact: true }).click()
      await expect(page.getByRole('alert').filter({ hasText: 'Keep it open' })).toBeVisible()
      await page.getByRole('button', { name: 'Settings', exact: true }).click()
    } else {
      await page.getByLabel('Learning language', { exact: true }).selectOption('sr')
      await expect(page.getByRole('alert').filter({ hasText: 'Keep it open' })).toBeVisible()
      await page.getByRole('button', { name: 'Nastavi vežbu', exact: true }).click()
    }
    await expect(page.getByRole('button', { name: 'My recording', exact: false })).toBeEnabled()
    await expect(page.getByRole('alert').filter({ hasText: 'Keep it open' })).toBeVisible()
    let sent: { id: string; hash: string } | null = null
    page.on('request', (request) => {
      if (!request.url().endsWith('/api/v1/recordings')) return
      const payload = request.postDataJSON()
      sent = {
        id: payload.id,
        hash: createHash('sha256')
          .update(Buffer.from(payload.audio_base64, 'base64'))
          .digest('hex'),
      }
    })
    await page.evaluate(() =>
      (window as unknown as { __restoreTakes: () => void }).__restoreTakes(),
    )
    await page.getByRole('button', { name: 'Retry', exact: true }).last().click()
    await expect
      .poll(
        async () =>
          (await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()).length,
      )
      .toBe(1)
    const recordings = await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()
    expect(sent).toEqual({ id: originalId, hash: recordings[0].sha256 })
    expect(recordings[0].id).toBe(originalId)
    await expect.poll(() => retainedKeys(page, 'take:')).toHaveLength(0)
  })
}
