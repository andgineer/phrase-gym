import { readFileSync } from 'node:fs'
import { test, expect } from '@playwright/test'

for (const action of ['stop', 'navigate']) {
  test(`pending reference playback is cancellable by ${action} without orphan players`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning', last_learning_language: 'en' },
    })
    const phrases = await (
      await request.get('/api/v1/phrases/search?language=en&q=greetings')
    ).json()
    const original = await (
      await request.get(`/api/v1/phrases/${phrases.items[0].id}?language=en&revision=1`)
    ).json()
    const fixture = JSON.parse(readFileSync('../fixtures/manual-en.json', 'utf8'))
    fixture.targets[0].reference_audio_id = original.content.reference_audio_id
    expect(fixture.targets[0].reference_audio_id).toBeTruthy()
    const staged = await (await request.post('/api/v1/imports', { data: fixture })).json()
    expect(staged.status).toBe('staged')
    expect(
      (
        await request.post(`/api/v1/imports/${staged.id}/publish`, { data: { reviewed: true } })
      ).ok(),
    ).toBe(true)
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
    await page.getByRole('button', { name: 'Back', exact: true }).click()
    await expect(page.getByRole('heading', { name: 'Library', exact: true })).toBeVisible()
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
