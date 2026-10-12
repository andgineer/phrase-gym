import { expect } from '@playwright/test'
import { test } from './capture-fixtures'
import { retainedAudio, syntheticCapture } from './synthetic-capture'

test.beforeEach(async ({ request }) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
})

test('the first recording keeps gaps through permission and capture, then reveals on stop', async ({
  page,
}) => {
  await syntheticCapture(page, true)
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect
    .poll(() =>
      page.evaluate(() =>
        Boolean((window as unknown as { __allowCapture?: () => void }).__allowCapture),
      ),
    )
    .toBe(true)
  await expect(page.getByTestId('practice-phrase')).toContainText('_____')
  await page.evaluate(() => (window as unknown as { __allowCapture: () => void }).__allowCapture())
  await expect(page.getByRole('button', { name: 'Stop recording', exact: true })).toBeVisible()
  await expect(page.getByTestId('practice-phrase')).toContainText('_____')
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).not.toContainText('_____')
  await expect
    .poll(() =>
      page.evaluate(
        () =>
          (window as unknown as { __testPlaybackVolumes: number[] }).__testPlaybackVolumes.length,
      ),
    )
    .toBeGreaterThan(0)
  expect(
    await page.evaluate(() =>
      (window as unknown as { __testPlaybackVolumes: number[] }).__testPlaybackVolumes.every(
        (volume) => volume === 0,
      ),
    ),
  ).toBe(true)
})

test('an oversized retained take cannot block a later valid recording', async ({
  page,
  request,
}) => {
  await syntheticCapture(page)
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const session = await (
    await request.get(`/api/v1/sessions/${page.url().split('/').pop()}`)
  ).json()
  const attempt = session.attempts[0]
  await page.evaluate(async (attempt) => {
    await new Promise<void>((resolve, reject) => {
      const opening = indexedDB.open('phrase-gym.practice.v1', 1)
      opening.onsuccess = () => {
        const transaction = opening.result.transaction('journal', 'readwrite')
        transaction.objectStore('journal').put(
          {
            id: '00000000-0000-4000-8000-000000000001',
            target_id: attempt.target_id,
            revision: attempt.revision,
            attempt_id: attempt.id,
            captured_at: new Date().toISOString(),
            mime_type: 'audio/wav',
            duration_ms: null,
            complete: false,
            capture_settings: {},
            audio: new ArrayBuffer(12 * 1024 * 1024 + 1),
            status: 'pending',
          },
          'take:00000000-0000-4000-8000-000000000001',
        )
        transaction.oncomplete = () => resolve()
        transaction.onerror = () => reject(transaction.error)
      }
    })
  }, attempt)
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBe(2)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect(page.getByRole('alert').first()).toContainText('at most 12 MiB')
  await expect
    .poll(
      async () =>
        (await (await request.get(`/api/v1/phrases/${attempt.target_id}/recordings`)).json())
          .length,
    )
    .toBe(1)
  await expect.poll(() => retainedAudio(page)).toBe(1)
  await page.reload()
  await expect(page.getByRole('alert').first()).toContainText('Audio remains on this device')
  await expect.poll(() => retainedAudio(page)).toBe(1)
})
