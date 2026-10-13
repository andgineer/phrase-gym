import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { downloadSession } from './offline-actions'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})
test('verified download reopens and completes offline, then reconciles exactly once', async ({
  page,
  connection,
  request,
}) => {
  const id = await downloadSession(page)
  connection.disconnect()
  await page.getByRole('button', { name: 'Show & explain' }).click()
  await expect(page.locator('.explanation')).toBeVisible()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.locator('.session-toolbar')).toContainText('2 / 3')
  await page.reload()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Pause', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Session paused' })).toBeVisible()
  await page.getByRole('button', { name: 'Resume session', exact: true }).last().click()
  await page.getByRole('button', { name: 'Not yet', exact: false }).click()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(
    page.getByRole('heading', { name: 'Finished on this device — awaiting confirmation' }),
  ).toBeVisible()
  await page.reload()
  await expect(
    page.getByRole('heading', { name: 'Finished on this device — awaiting confirmation' }),
  ).toBeVisible()
  connection.reconnect()
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect(page.getByRole('heading', { name: 'Session complete' })).toBeVisible()
  const state = await (await request.get(`/api/v1/sessions/${id}`)).json()
  expect(state.attempts).toHaveLength(3)
  expect(state.attempts[0].original.supported).toBe(true)
  expect(state.status).toBe('completed')
  await page.reload()
  const again = await (await request.get(`/api/v1/sessions/${id}`)).json()
  expect(again.last_sequence).toBe(state.last_sequence)
})

test('missing audio never claims a verified download', async ({ page, request }) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.evaluate(async () => {
    await navigator.serviceWorker.ready
  })
  await expect
    .poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller)))
    .toBe(true)
  const session = await (
    await request.get(`/api/v1/sessions/${page.url().split('/').pop()}`)
  ).json()
  await request.post(`/_test/missing-media/${session.manifest.required_media_ids[0]}`)
  await page.getByRole('button', { name: 'Download session' }).click()
  await expect(page.getByRole('alert').filter({ hasText: 'Download incomplete' })).toBeVisible()
  await expect(page.getByText('Verified for offline practice')).toHaveCount(0)
})
test('quota failure never claims a verified download', async ({ page }) => {
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await page.evaluate(async () => {
    await navigator.serviceWorker.ready
  })
  await expect
    .poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller)))
    .toBe(true)
  await page.evaluate(() => {
    Cache.prototype.put = async () => {
      throw new DOMException('Full', 'QuotaExceededError')
    }
  })
  await page.getByRole('button', { name: 'Download session' }).click()
  await expect(page.getByRole('alert').filter({ hasText: 'QuotaExceededError' })).toBeVisible()
  await expect(page.getByText('Verified for offline practice')).toHaveCount(0)
})

test('evicted selected audio blocks offline practice with recovery guidance', async ({
  page,
  connection,
}) => {
  await downloadSession(page)
  await page.evaluate(() => caches.delete('phrase-gym.selected-media.v1'))
  connection.disconnect()
  await page.reload()
  await expect(page.getByRole('alert').filter({ hasText: 'evicted' }).first()).toBeVisible()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeDisabled()
  await expect(page.getByRole('heading', { name: 'Finished on this device' })).toHaveCount(0)
})

test('offline reference uses the pinned cached bytes and handles range playback', async ({
  page,
  connection,
}) => {
  await downloadSession(page)
  const media = await page.evaluate(async () => {
    const cache = await caches.open('phrase-gym.selected-media.v1')
    return (await cache.keys())[0]!.url
  })
  connection.disconnect()
  const response = await page.evaluate(async (url) => {
    const r = await fetch(url, { headers: { Range: 'bytes=0-9' } })
    return { status: r.status, length: (await r.arrayBuffer()).byteLength }
  }, media)
  expect(response).toEqual({ status: 206, length: 10 })
  await page.getByRole('button', { name: 'Reference', exact: false }).click()
  await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeVisible()
})

test('an evicted shell is repaired by a new verified download and packs remain reachable from home', async ({
  page,
  connection,
}) => {
  await downloadSession(page)
  await page.evaluate(async () => {
    for (const name of await caches.keys())
      if (name.startsWith('phrase-gym.shell.')) await caches.delete(name)
  })
  await page.getByRole('button', { name: 'Download session', exact: true }).click()
  await expect(
    page.getByRole('status').filter({ hasText: 'Verified for offline practice' }),
  ).toBeVisible()
  connection.disconnect()
  await page.goto('/learn/en')
  await expect(page.getByRole('heading', { name: 'Downloaded sessions' })).toBeVisible()
  await page.getByRole('link', { name: /Resume session/ }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
})

test('a failed event journal write never advances or claims a locally saved rating', async ({
  page,
  request,
}) => {
  const id = await downloadSession(page)
  await page.evaluate(() => {
    const put = IDBObjectStore.prototype.put
    IDBObjectStore.prototype.put = function (value, key) {
      if (String(key).startsWith('event:')) throw new DOMException('Full', 'QuotaExceededError')
      return put.call(this, value, key)
    }
    Object.assign(window, {
      __restoreJournal: () => {
        IDBObjectStore.prototype.put = put
      },
    })
  })
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(
    page.getByRole('alert').filter({ hasText: 'QuotaExceededError' }).first(),
  ).toBeVisible()
  await expect(page.locator('.session-toolbar')).toContainText('1 / 3')
  const state = await (await request.get(`/api/v1/sessions/${id}`)).json()
  expect(state.attempts.some((a: { rating: string }) => a.rating)).toBe(false)
  await page.evaluate(() =>
    (window as unknown as { __restoreJournal: () => void }).__restoreJournal(),
  )
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.locator('.session-toolbar')).toContainText('2 / 3')
})

test('offline cues retain keyboard focus and readable enlarged layout in both color schemes', async ({
  page,
  connection,
}) => {
  await page.setViewportSize({ width: 320, height: 740 })
  await downloadSession(page)
  await page.evaluate(() => {
    document.documentElement.style.fontSize = '24px'
  })
  connection.disconnect()
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.locator('.session-toolbar')).toContainText('2 / 3')
  await expect(page.getByTestId('practice-phrase')).toBeFocused()
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(
    true,
  )
  for (const colorScheme of ['light', 'dark'] as const) {
    await page.emulateMedia({ colorScheme })
    const size = await page.getByRole('button', { name: 'Record', exact: true }).boundingBox()
    expect(size!.width).toBeGreaterThanOrEqual(44)
    expect(size!.height).toBeGreaterThanOrEqual(44)
    await expect(page.getByTestId('practice-phrase')).toHaveAttribute('lang', 'en')
  }
})
