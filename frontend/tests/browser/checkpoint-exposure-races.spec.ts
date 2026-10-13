import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession, readCheckpoint } from './offline-actions'

test('a delayed equal-version read preserves another tab lookup through offline completion', async ({
  page,
  context,
  request,
  connection,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  const id = await downloadSession(page)
  const before = await readCheckpoint(page, id)
  const target = before.attempts[0]!.target_id
  expect(before.support_exposures?.[target]).toEqual([])
  await request.post(`/_test/hold-session-read/${id}`)
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect
    .poll(async () => (await (await request.get('/_test/session-read-state')).json()).held)
    .toBe(true)
  const second = await context.newPage()
  // Distinct cache keys prevent Chromium from coalescing the two held API reads.
  await second.addInitScript((id) => {
    const fetchOriginal = window.fetch.bind(window)
    window.fetch = (input, options) =>
      fetchOriginal(
        input === `/api/v1/sessions/${id}` ? `${input}?read-observer=second` : input,
        options,
      )
  }, id)
  try {
    await second.goto(`/learn/en/phrases/${target}`)
    await expect
      .poll(
        async () =>
          (await (await request.get(`/api/v1/sessions/${id}`)).json()).support_exposures[target]
            .length,
      )
      .toBe(1)
    await second.goto(`/learn/en/practice/${id}`)
    await expect(second.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    const richer = await readCheckpoint(second, id)
    expect(richer.last_sequence).toBe(before.last_sequence)
    expect(richer.version).toBe(before.version)
    expect(richer.support_exposures?.[target]).toHaveLength(1)
    await request.post('/_test/release-session-read')
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    expect((await readCheckpoint(page, id)).support_exposures?.[target]).toEqual(
      richer.support_exposures?.[target],
    )
    await page.close()
    connection.disconnect()
    await second.reload()
    for (let i = 0; i < before.queue.length; i++) {
      await expect(second.getByRole('button', { name: 'Recalled' })).toBeEnabled()
      await second.getByRole('button', { name: 'Recalled' }).click()
    }
    await expect(
      second.getByRole('heading', { name: 'Finished on this device — awaiting confirmation' }),
    ).toBeVisible()
    await expect(second.locator('.practice-panel li').first()).toContainText('With help')
    await second.reload()
    await expect(second.locator('.practice-panel li').first()).toContainText('With help')
    connection.reconnect()
    await second.evaluate(() => window.dispatchEvent(new Event('online')))
    await expect(
      second.getByRole('heading', { name: 'Session complete', exact: true }),
    ).toBeVisible()
    const saved = await (await request.get(`/api/v1/sessions/${id}`)).json()
    expect(saved.attempts[0].original.supported).toBe(true)
    await expect(second.locator('.practice-panel li').first()).toContainText('With help')
  } finally {
    await request.post('/_test/release-session-read')
    await second.close()
  }
})
