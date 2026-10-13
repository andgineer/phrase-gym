import { expect } from '@playwright/test'
import { test } from './connection-fixtures'
import { startSession, retainedKeys, readCheckpoint } from './session-actions'
import { syntheticCapture, retainedAudio } from './synthetic-capture'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('a lost committed rating hides practice until original-ID retry confirms progress', async ({
  page,
  request,
}) => {
  const id = await startSession(page)
  const before = await readCheckpoint(page, id)
  await request.post('/_test/lose-rating')
  await page.getByRole('button', { name: 'Recalled' }).click()
  await expect(page.getByTestId('connection-required')).toBeVisible()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeHidden()
  expect((await readCheckpoint(page, id)).cursor).toBe(before.cursor)
  const keys = await retainedKeys(page, 'event:')
  expect(keys).toHaveLength(1)
  const committed = await (await request.get(`/api/v1/sessions/${id}`)).json()
  expect(committed.cursor).toBe(1)
  await request.post('/_test/acknowledge-rating')
  await page
    .getByTestId('connection-required')
    .getByRole('button', { name: 'Retry', exact: true })
    .click()
  await expect(page.getByTestId('connection-required')).toHaveCount(0)
  await expect(page.locator('.session-toolbar')).toContainText('2 / 3')
  await expect.poll(() => retainedKeys(page, 'event:')).toHaveLength(0)
  const receipts = await (await request.get(`/api/v1/sessions/${id}/events`)).json()
  expect(receipts.filter((r: { id: string }) => `event:${r.id}` === keys[0])).toHaveLength(1)
  expect(receipts.filter((r: { schedule_changed: boolean }) => r.schedule_changed)).toHaveLength(1)
})

for (const mode of ['capture', 'permission', 'playback'])
  test(`server loss keeps active ${mode} stoppable without new learning`, async ({
    page,
    connection,
    request,
  }) => {
    await syntheticCapture(page, mode === 'permission')
    const id = await startSession(page)
    if (mode === 'playback') {
      await page.evaluate(() => {
        HTMLMediaElement.prototype.play = async function () {
          this.dispatchEvent(new Event('playing'))
        }
        const pause = HTMLMediaElement.prototype.pause
        HTMLMediaElement.prototype.pause = function () {
          Object.assign(window, { __stoppedPlayback: true })
          pause.call(this)
        }
      })
      await page.getByRole('button', { name: 'Reference' }).click()
      await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeEnabled()
    } else {
      await page.getByRole('button', { name: 'Record', exact: true }).click()
      if (mode === 'capture') await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
      else
        await expect(
          page.getByRole('button', { name: 'Cancel microphone request', exact: true }),
        ).toBeEnabled()
    }
    if (mode === 'permission')
      await expect
        .poll(() =>
          page.evaluate(() =>
            Boolean((window as unknown as { __allowCapture?: () => void }).__allowCapture),
          ),
        )
        .toBe(true)
    const before = await (await request.get(`/api/v1/sessions/${id}`)).json()
    connection.disconnect()
    await page.evaluate(() => window.dispatchEvent(new Event('offline')))
    await expect(page.getByTestId('connection-required')).toBeVisible()
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeHidden()
    await expect(page.getByRole('button', { name: 'Download session' })).toHaveCount(0)
    await page.getByRole('button', { name: 'Stop or cancel audio', exact: true }).click()
    if (mode === 'permission') {
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
    } else if (mode === 'playback') {
      expect(
        await page.evaluate(
          () => (window as unknown as { __stoppedPlayback: boolean }).__stoppedPlayback,
        ),
      ).toBe(true)
    } else {
      await expect.poll(() => retainedKeys(page, 'event:')).toHaveLength(1)
      await expect.poll(() => retainedAudio(page)).toBe(1)
    }
    const unchanged = await (await request.get(`/api/v1/sessions/${id}`)).json()
    expect(unchanged.cursor).toBe(before.cursor)
    expect(unchanged.last_sequence).toBe(before.last_sequence)
    connection.reconnect()
    await page
      .getByTestId('connection-required')
      .getByRole('button', { name: 'Retry', exact: true })
      .click()
    await expect(page.getByTestId('connection-required')).toHaveCount(0)
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    if (mode === 'capture') {
      await expect.poll(() => retainedAudio(page)).toBe(0)
      const state = await (await request.get(`/api/v1/sessions/${id}`)).json()
      const takes = await (
        await request.get(`/api/v1/phrases/${state.attempts[0].target_id}/recordings`)
      ).json()
      expect(takes).toHaveLength(1)
      expect(takes[0].complete).toBe(true)
      expect(state.attempts[0].original.modality).toBe('recorded')
    }
  })
