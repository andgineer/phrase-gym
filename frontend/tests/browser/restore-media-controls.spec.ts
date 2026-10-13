import { expect } from '@playwright/test'
import { test } from './capture-fixtures'
import { syntheticCapture, retainedAudio } from './synthetic-capture'

for (const mode of ['capture', 'permission', 'playback'])
  test(`restoration blocks new actions but preserves ${mode} stop or cancel`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    await syntheticCapture(page, mode === 'permission')
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Start session', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    const id = page.url().split('/').pop()!
    if (mode === 'playback') {
      await page.evaluate(() => {
        const pause = HTMLMediaElement.prototype.pause
        HTMLMediaElement.prototype.play = async function () {
          this.dispatchEvent(new Event('playing'))
        }
        HTMLMediaElement.prototype.pause = function () {
          Object.assign(window, { __stoppedPlayback: true })
          pause.call(this)
        }
      })
      await page.getByRole('button', { name: 'Reference' }).click()
    } else {
      await page.getByRole('button', { name: 'Record', exact: true }).click()
      if (mode === 'capture') await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
      else
        await expect
          .poll(() =>
            page.evaluate(() =>
              Boolean((window as unknown as { __allowCapture?: () => void }).__allowCapture),
            ),
          )
          .toBe(true)
    }
    const label =
      mode === 'capture'
        ? 'Stop recording'
        : mode === 'permission'
          ? 'Cancel microphone request'
          : 'Stop playback'
    await expect(page.getByRole('button', { name: label, exact: true })).toBeEnabled()
    let held = false
    let release!: () => void
    const gate = new Promise<void>((resolve) => {
      release = resolve
    })
    await page.route('**/api/v1/sessions/current', async (route) => {
      const response = await route.fetch()
      held = true
      await gate
      await route.fulfill({ response })
    })
    await page.evaluate(() => window.dispatchEvent(new Event('online')))
    await expect.poll(() => held).toBe(true)
    try {
      await expect(page.getByRole('button', { name: 'Recalled' })).toBeDisabled()
      await expect(page.getByRole('button', { name: label, exact: true })).toBeEnabled()
      await page.getByRole('button', { name: label, exact: true }).click()
      await expect(page.getByRole('button', { name: label, exact: true })).toHaveCount(0)
      if (mode === 'capture') {
        const session = await (await request.get(`/api/v1/sessions/${id}`)).json()
        await expect
          .poll(
            async () =>
              (
                await (
                  await request.get(`/api/v1/phrases/${session.attempts[0].target_id}/recordings`)
                ).json()
              ).length,
          )
          .toBe(1)
        const takes = await (
          await request.get(`/api/v1/phrases/${session.attempts[0].target_id}/recordings`)
        ).json()
        expect(takes[0].complete).toBe(true)
      } else if (mode === 'permission') {
        await page.evaluate(() =>
          (window as unknown as { __allowCapture: () => void }).__allowCapture(),
        )
        await expect
          .poll(() =>
            page.evaluate(
              () => (window as unknown as { __captureTrackState: string }).__captureTrackState,
            ),
          )
          .toBe('ended')
      } else
        expect(
          await page.evaluate(
            () => (window as unknown as { __stoppedPlayback: boolean }).__stoppedPlayback,
          ),
        ).toBe(true)
    } finally {
      release()
    }
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    const saved = await (await request.get(`/api/v1/sessions/${id}`)).json()
    expect(saved.attempts[0].original?.modality ?? null).toBe(
      mode === 'playback' ? null : 'recorded',
    )
  })

test('an authoritative card change releases old capture without replay on the next target', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await syntheticCapture(page)
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const id = page.url().split('/').pop()!
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  const state = await (await request.get(`/api/v1/sessions/${id}`)).json()
  const target = state.attempts[0].target_id
  const replacement = await request.post(`/api/v1/sessions/${id}/events`, {
    data: {
      events: [
        {
          id: crypto.randomUUID(),
          sequence: state.last_sequence + 1,
          step: state.cursor,
          occurred_at: new Date().toISOString(),
          kind: 'rate',
          rating: 'recalled',
          modality: 'self_report',
        },
      ],
    },
  })
  expect(replacement.ok()).toBe(true)
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
  await expect(page.locator('.session-toolbar')).toContainText('2 / 3')
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as { __captureTrackState: string }).__captureTrackState,
      ),
    )
    .toBe('ended')
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await expect(page.getByRole('button', { name: 'My recording' })).toBeDisabled()
  await expect
    .poll(
      async () => (await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()).length,
    )
    .toBe(1)
  const takes = await (await request.get(`/api/v1/phrases/${target}/recordings`)).json()
  expect(takes[0].complete).toBe(false)
  const saved = await (await request.get(`/api/v1/sessions/${id}`)).json()
  expect(saved.attempts[0].original.modality).toBe('recorded')
  expect(saved.attempts[1].technical_failure).toBe(false)
})

test('lookup permission remains cancellable and releases its late stream without a session', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await syntheticCapture(page, true)
  await page.goto('/learn/en?q=greetings')
  await page.getByRole('link', { name: /How has your day/ }).click()
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect
    .poll(() =>
      page.evaluate(() =>
        Boolean((window as unknown as { __allowCapture?: () => void }).__allowCapture),
      ),
    )
    .toBe(true)
  await page.getByRole('button', { name: 'Cancel microphone request', exact: true }).click()
  await page.evaluate(() => (window as unknown as { __allowCapture: () => void }).__allowCapture())
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as { __captureTrackState: string }).__captureTrackState,
      ),
    )
    .toBe('ended')
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  expect(await (await request.get('/api/v1/sessions/current')).json()).toBeNull()
  expect(await retainedAudio(page)).toBe(0)
})
