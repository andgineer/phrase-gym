import { expect } from '@playwright/test'
import type { Page } from '@playwright/test'
import { test } from './fixtures'

async function capturePlayers(page: Page) {
  await page.addInitScript(() => {
    const original = window.Audio
    const players: HTMLAudioElement[] = []
    Object.defineProperty(window, 'referencePlayers', { value: players })
    window.Audio = function (src?: string) {
      const player = new original(src)
      players.push(player)
      return player
    } as unknown as typeof Audio
  })
}

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

for (const action of ['stop', 'navigate', 'background', 'settings']) {
  test(`pending reference playback is cancellable by ${action} without orphan players`, async ({
    page,
  }) => {
    await capturePlayers(page)
    await page.goto('/learn/en?q=greetings')
    await page.getByRole('link', { name: /How has your day/ }).click()
    let release!: () => void
    const gate = new Promise<void>((resolve) => {
      release = resolve
    })
    let mediaRequests = 0
    await page.route(/\/api\/v1\/media\/[^/]+$/, async (route) => {
      mediaRequests += 1
      await gate
      await route.continue()
    })
    await page.getByRole('button', { name: 'Reference', exact: true }).click()
    await expect.poll(() => mediaRequests).toBe(1)
    await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
    if (action === 'stop') {
      await page.getByRole('button', { name: 'Stop playback', exact: true }).click()
      await expect(page.getByRole('button', { name: 'Reference', exact: true })).toBeVisible()
    }
    if (action === 'background') {
      await page.evaluate(() => {
        Object.defineProperty(document, 'hidden', { configurable: true, value: true })
        document.dispatchEvent(new Event('visibilitychange'))
      })
      await expect(page.getByRole('button', { name: 'Reference', exact: true })).toBeVisible()
    }
    if (action === 'settings') {
      await page.getByRole('button', { name: 'Settings', exact: true }).click()
      await expect(page.getByRole('heading', { name: 'Settings', exact: true })).toBeVisible()
    } else {
      await page.getByRole('button', { name: 'Back', exact: true }).click()
      await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
    }
    release()
    const state = await page.evaluate(() => {
      const players = (window as unknown as { referencePlayers: HTMLAudioElement[] })
        .referencePlayers
      return players.map((player) => ({ paused: player.paused, src: player.getAttribute('src') }))
    })
    expect(state).toEqual([{ paused: true, src: '' }])
    await expect(page.getByRole('alert')).toHaveCount(0)
    expect(mediaRequests).toBe(1)
  })
}

test('failed reference delivery offers a working explicit playback retry', async ({ page }) => {
  await capturePlayers(page)
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await page.route(/\/api\/v1\/media\/[^/]+$/, (route) =>
    route.fulfill({ status: 503, body: 'Unavailable' }),
  )
  await page.getByRole('button', { name: 'Reference', exact: true }).click()
  await expect(page.getByRole('alert')).toContainText('Audio could not play')
  await expect(page.getByRole('button', { name: 'Reference', exact: true })).toBeVisible()
  await page.unroute(/\/api\/v1\/media\/[^/]+$/)
  await page.getByRole('button', { name: 'Reference', exact: true }).click()
  await expect
    .poll(() =>
      page.evaluate(() => {
        const players = (window as unknown as { referencePlayers: HTMLAudioElement[] })
          .referencePlayers
        return players.length === 2 && !players[1].paused
      }),
    )
    .toBe(true)
  await expect(page.getByRole('alert')).toHaveCount(0)
  await page.getByRole('button', { name: 'Stop playback', exact: true }).click()
  expect(
    await page.evaluate(() =>
      (window as unknown as { referencePlayers: HTMLAudioElement[] }).referencePlayers.every(
        (player) => player.paused && player.getAttribute('src') === '',
      ),
    ),
  ).toBe(true)
})

test('Settings hides materials and cancels active Reference playback', async ({ page }) => {
  await capturePlayers(page)
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await page.getByRole('button', { name: 'Reference', exact: true }).click()
  await expect
    .poll(() =>
      page.evaluate(() =>
        (window as unknown as { referencePlayers: HTMLAudioElement[] }).referencePlayers.some(
          (player) => !player.paused,
        ),
      ),
    )
    .toBe(true)
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await expect(page.getByRole('heading', { name: 'Settings', exact: true })).toBeVisible()
  await expect(page.locator('.phrase-card')).toHaveCount(0)
  expect(
    await page.evaluate(() =>
      (window as unknown as { referencePlayers: HTMLAudioElement[] }).referencePlayers.every(
        (player) => player.paused && player.getAttribute('src') === '',
      ),
    ),
  ).toBe(true)
  await page.getByRole('button', { name: 'Settings', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Reference', exact: true })).toBeVisible()
  await expect(page.getByRole('alert')).toHaveCount(0)
})
