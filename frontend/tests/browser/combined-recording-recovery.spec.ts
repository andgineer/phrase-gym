import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession, retainedKeys } from './offline-actions'
import { syntheticCapture } from './synthetic-capture'
import { readTake } from './take-race-helpers'

for (const dialogue of [false, true])
  test(`interrupted quota audio survives navigation, another tab, completion and a waiting update, dialogue=${dialogue}`, async ({
    page,
    context,
    request,
    connection,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    await syntheticCapture(page)
    let id: string
    if (dialogue) {
      await request.post('/_test/two-target-dialogue')
      const prepared = (await (await request.get('/api/v1/dialogues?language=en')).json()).items[0]
      const state = await (
        await request.post('/api/v1/sessions', {
          data: { id: crypto.randomUUID(), language: 'en', dialogue_id: prepared.id },
        })
      ).json()
      id = state.id
      await page.goto(`/learn/en/practice/${id}`)
      await page.getByRole('button', { name: 'Rehearse dialogue', exact: true }).click()
      await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
      await page.evaluate(() => navigator.serviceWorker.ready)
      await expect
        .poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller)))
        .toBe(true)
      await page.getByRole('button', { name: 'Download session', exact: true }).click()
      await expect(page.getByText('Verified for offline practice')).toBeVisible()
    } else id = await downloadSession(page)
    const initial = await (await request.get(`/api/v1/sessions/${id}`)).json()
    const target = initial.attempts[0].target_id
    await page.evaluate(async () => navigator.serviceWorker.register('/sw.js?combined-update=1'))
    await expect
      .poll(() =>
        page.evaluate(async () =>
          Boolean((await navigator.serviceWorker.getRegistration())?.waiting),
        ),
      )
      .toBe(true)
    await page.evaluate(() => {
      const put = IDBObjectStore.prototype.put
      IDBObjectStore.prototype.put = function (value, key) {
        if (String(key).startsWith('take-v2:') && value.audio?.byteLength)
          throw new DOMException('Full', 'QuotaExceededError')
        return put.call(this, value, key)
      }
      Object.assign(window, {
        __restoreTakes: () => {
          IDBObjectStore.prototype.put = put
        },
      })
    })
    connection.disconnect()
    await page.getByRole('button', { name: 'Record', exact: true }).click()
    await expect(page.getByRole('alert').filter({ hasText: 'Keep it open' })).toBeVisible()
    const takeId = (await retainedKeys(page, 'take-v2:'))[0]!.split(':').pop()!
    if (dialogue) await page.getByLabel('Learning language', { exact: true }).selectOption('sr')
    else await page.getByRole('button', { name: 'Settings', exact: true }).click()
    await expect
      .poll(() =>
        page.evaluate(
          () => (window as unknown as { __captureTrackState: string }).__captureTrackState,
        ),
      )
      .toBe('ended')
    await expect(page.getByRole('alert').filter({ hasText: 'Keep it open' })).toBeVisible()
    const reader = await context.newPage()
    await reader.goto(`/learn/en/practice/${id}`)
    await reader.bringToFront()
    await reader.evaluate(() => window.dispatchEvent(new Event('online')))
    await expect(reader.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    expect(await readTake(reader, takeId)).toMatchObject({ status: 'capturing', size: 0 })
    expect(await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()).toHaveLength(0)
    await reader.close()
    await page.bringToFront()
    if (dialogue) await page.getByRole('button', { name: 'Nastavi vežbu', exact: true }).click()
    else await page.getByRole('button', { name: 'Settings', exact: true }).click()
    await expect(page.getByRole('button', { name: 'My recording' })).toBeEnabled()
    for (let index = 0; index < initial.queue.length; index++) {
      await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
      await page.getByRole('button', { name: 'Recalled' }).click()
    }
    await expect(
      page.getByRole('heading', { name: 'Finished on this device — awaiting confirmation' }),
    ).toBeVisible()
    await expect(page.getByRole('button', { name: 'Update app', exact: true })).toBeDisabled()
    await page.evaluate(() =>
      (window as unknown as { __restoreTakes: () => void }).__restoreTakes(),
    )
    connection.reconnect()
    await page.evaluate(() => document.dispatchEvent(new Event('visibilitychange')))
    await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
    await expect.poll(() => retainedKeys(page, 'take-v2:')).toHaveLength(0)
    const takes = await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()
    expect(takes).toHaveLength(1)
    expect(takes[0]).toMatchObject({ id: takeId, complete: false })
    const saved = await (await request.get(`/api/v1/sessions/${id}`)).json()
    expect(saved.attempts[0].original).toMatchObject({ modality: 'recorded', supported: dialogue })
    expect(saved.attempts[0].eligibility_reason).toBe('technical_failure')
    await expect(page.getByRole('button', { name: 'Update app', exact: true })).toBeEnabled()
    await Promise.all([
      page.waitForEvent('load'),
      page.getByRole('button', { name: 'Update app', exact: true }).click(),
    ])
    await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
    expect(await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()).toEqual(takes)
  })
