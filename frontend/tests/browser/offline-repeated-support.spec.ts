import { expect } from '@playwright/test'
import { test } from './offline-fixtures'
import { installEventClock, setEventClock } from './event-clock'

for (const acknowledgedHelp of [false, true])
  test(`expired on-card help is not refreshed by rating a repeated target, acknowledged=${acknowledgedHelp}`, async ({
    page,
    context,
    request,
    connection,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    await request.post('/_test/repeated-target-dialogue')
    await installEventClock(context)
    const dialogue = (await (await request.get('/api/v1/dialogues?language=en')).json()).items[0]
    const session = await (
      await request.post('/api/v1/sessions', {
        data: { id: crypto.randomUUID(), language: 'en', dialogue_id: dialogue.id },
      })
    ).json()
    await page.goto(`/learn/en/practice/${session.id}`)
    await page.getByRole('button', { name: 'Rehearse dialogue', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    await page.evaluate(() => navigator.serviceWorker.ready)
    await expect
      .poll(() => page.evaluate(() => Boolean(navigator.serviceWorker.controller)))
      .toBe(true)
    if (acknowledgedHelp) await page.getByRole('button', { name: 'Show & explain' }).click()
    await page.getByRole('button', { name: 'Download session', exact: true }).click()
    await expect(page.getByText('Verified for offline practice')).toBeVisible()
    connection.disconnect()
    if (!acknowledgedHelp) await page.getByRole('button', { name: 'Show & explain' }).click()
    await expect(page.locator('.explanation')).toBeVisible()
    await request.post('/_test/advance-clock/360')
    await setEventClock(page, 360)
    await page.reload()
    await page.getByRole('button', { name: 'Recalled' }).click()
    await expect(page.locator('.session-toolbar')).toContainText('2 / 2')
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    await expect(page.getByRole('status').filter({ hasText: 'With help' })).toHaveCount(0)
    await page.reload()
    await expect(page.getByRole('status').filter({ hasText: 'With help' })).toHaveCount(0)
    await page.getByRole('button', { name: 'Recalled' }).click()
    await expect(
      page.getByRole('heading', { name: 'Finished on this device — awaiting confirmation' }),
    ).toBeVisible()
    const results = page.locator('.practice-panel li')
    await expect(results.nth(0)).toContainText('With help')
    await expect(results.nth(1)).not.toContainText('With help')
    connection.reconnect()
    await page.evaluate(() => document.dispatchEvent(new Event('visibilitychange')))
    await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
    await expect(results.nth(1)).not.toContainText('With help')
    const saved = await (await request.get(`/api/v1/sessions/${session.id}`)).json()
    expect(saved.attempts[1].original.supported).toBe(false)
    expect(saved.attempts[1].eligibility_reason).toBe('review_group_already_finalized')
    expect(
      saved.support_exposures[saved.attempts[0].target_id].every(
        (time: string) => Date.parse(time) < Date.parse(saved.attempts[0].finalized_at),
      ),
    ).toBe(true)
  })
