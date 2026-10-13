import { expect } from '@playwright/test'
import { test } from './offline-fixtures'

for (const offlineStart of [false, true])
  for (const seconds of [240, 360])
    test(`dialogue script support expires consistently after ${seconds}s, offline start=${offlineStart}`, async ({
      page,
      context,
      request,
      connection,
    }) => {
      await request.put('/api/v1/settings', {
        data: { native_language: 'ru', ui_language_mode: 'learning' },
      })
      await request.post('/_test/two-target-dialogue')
      await context.addInitScript(() => {
        const NativeDate = Date
        window.Date = new Proxy(NativeDate, {
          construct(target, args) {
            const offset = Number(localStorage.getItem('synthetic-clock-offset') ?? 0)
            return Reflect.construct(target, args.length ? args : [target.now() + offset])
          },
          get(target, key) {
            return key === 'now'
              ? () => target.now() + Number(localStorage.getItem('synthetic-clock-offset') ?? 0)
              : Reflect.get(target, key)
          },
        })
      })
      const dialogue = (await (await request.get('/api/v1/dialogues?language=en')).json()).items[0]
      const response = await request.post('/api/v1/sessions', {
        data: { id: crypto.randomUUID(), language: 'en', dialogue_id: dialogue.id },
      })
      expect(response.ok()).toBe(true)
      const session = await response.json()
      await page.goto(`/learn/en/practice/${session.id}`)
      await expect(
        page.getByRole('button', { name: 'Rehearse dialogue', exact: true }),
      ).toBeVisible()
      await page.evaluate(() => navigator.serviceWorker.ready)
      await expect
        .poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller)))
        .toBe(true)
      if (!offlineStart)
        await page.getByRole('button', { name: 'Rehearse dialogue', exact: true }).click()
      await page.getByRole('button', { name: 'Download session', exact: true }).click()
      await expect(page.getByText('Verified for offline practice')).toBeVisible()
      if (!offlineStart) {
        const checkpoint = await (await request.get(`/api/v1/sessions/${session.id}`)).json()
        expect(checkpoint.dialogue_exposures.length).toBeGreaterThan(0)
      }
      connection.disconnect()
      if (offlineStart)
        await page.getByRole('button', { name: 'Rehearse dialogue', exact: true }).click()
      await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
      await expect(page.getByRole('status').filter({ hasText: 'With help' })).toBeVisible()
      await request.post(`/_test/advance-clock/${seconds}`)
      await page.evaluate(
        (offset) => localStorage.setItem('synthetic-clock-offset', String(offset)),
        seconds * 1000,
      )
      await page.reload()
      await page.getByRole('button', { name: 'Recalled' }).click()
      await expect(page.locator('.session-toolbar')).toContainText('2 / 2')
      await page.reload()
      await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
      await expect(page.getByRole('status').filter({ hasText: 'With help' })).toHaveCount(
        seconds === 240 ? 1 : 0,
      )
      await page.getByRole('button', { name: 'Recalled' }).click()
      await expect(
        page.getByRole('heading', { name: 'Finished on this device — awaiting confirmation' }),
      ).toBeVisible()
      const result = page
        .locator('.practice-panel li')
        .filter({ hasText: 'I’d rather meet tomorrow.' })
      if (seconds === 240) await expect(result).toContainText('With help')
      else await expect(result).not.toContainText('With help')
      const provisional = await result.innerText()
      await page.reload()
      await expect(result).toHaveText(provisional)
      connection.reconnect()
      await page.evaluate(() => window.dispatchEvent(new Event('online')))
      await expect(
        page.getByRole('heading', { name: 'Session complete', exact: true }),
      ).toBeVisible()
      await expect(result).toHaveText(provisional)
      const confirmed = await (await request.get(`/api/v1/sessions/${session.id}`)).json()
      expect(confirmed.attempts[0].original.supported).toBe(true)
      expect(confirmed.attempts[1].original.supported).toBe(seconds === 240)
      expect(confirmed.attempts[1].eligibility_reason).toBe(
        seconds === 240 ? 'supported_learning_check' : 'eligible',
      )
      await page.reload()
      const replay = await (await request.get(`/api/v1/sessions/${session.id}`)).json()
      expect(replay.last_sequence).toBe(confirmed.last_sequence)
      expect(replay.attempts).toEqual(confirmed.attempts)
    })
