import { card, extraPackage } from './extra-package'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { importPackage, revealAnswer, savedTakes } from './helpers'

test.beforeEach(async ({ page }) => {
  await installFakeMedia(page)
  await page.goto('/')
  await importPackage(page, extraPackage, 3)
  await page.getByRole('button', { name: 'Start', exact: true }).click()
  await revealAnswer(page)
})

test('recording save warnings count failures and clear after a successful retry', async ({
  page,
}) => {
  const practiceCard = card(page)
  const phrase = await practiceCard.getByTestId('practice-phrase').innerText()
  const saveFailure = practiceCard
    .getByRole('alert')
    .filter({ hasText: 'Audio remains on this device.' })
  const saving = practiceCard.getByRole('status').filter({ hasText: 'Saving recording…' })
  const retry = practiceCard.getByRole('button', { name: 'Retry save' })
  const record = practiceCard.getByRole('button', { name: 'Record a take' })
  const stop = practiceCard.getByRole('button', { name: 'Stop recording' })
  const stopPlayback = practiceCard.getByRole('button', { name: 'Stop playback' })

  let releaseFirst!: () => void
  const firstHeld = new Promise<void>((resolve) => (releaseFirst = resolve))
  let releaseRetries!: () => void
  const retriesHeld = new Promise<void>((resolve) => (releaseRetries = resolve))
  const failedIds = new Set<string>()
  const retriedIds: string[] = []
  let firstRequested = false
  let allowSave = false
  await page.route('**/api/recordings', async (route) => {
    const request = route.request()
    if (request.method() !== 'POST') return route.continue()
    const { id } = request.postDataJSON() as { id: string }
    if (allowSave) {
      retriedIds.push(id)
      await retriesHeld
      return route.continue()
    }
    if (!firstRequested) {
      firstRequested = true
      await firstHeld
    }
    failedIds.add(id)
    return route.fulfill({ status: 503, json: { detail: 'Temporary save failure' } })
  })

  await record.click()
  await expect(stop).toBeVisible()
  await expect(saveFailure).toHaveCount(0)
  await expect(saving).toHaveCount(0)
  await expect(retry).toHaveCount(0)
  await stop.click()
  await expect.poll(() => firstRequested).toBe(true)
  await expect(saveFailure).toHaveCount(0)
  await expect(saving).toHaveCount(0)
  await expect(retry).toHaveCount(0)
  releaseFirst()

  await expect(saveFailure).toContainText('1 recording could not be saved.')
  await expect(retry).toBeEnabled()
  await stopPlayback.click()
  await record.click()
  await expect(stop).toBeVisible()
  await expect(saveFailure).toContainText('1 recording could not be saved.')
  await expect(retry).toBeEnabled()
  await stop.click()
  await expect(saveFailure).toContainText('2 recordings could not be saved.')
  expect(failedIds.size).toBe(2)
  expect(await savedTakes(page, phrase)).toEqual([])

  allowSave = true
  await retry.click()
  await expect.poll(() => retriedIds.length).toBe(1)
  await expect(saving).toBeVisible()
  await expect(saveFailure).toContainText('1 recording could not be saved.')
  await expect(retry).toHaveCount(0)
  releaseRetries()
  await expect(saving).toHaveCount(0)
  await expect(saveFailure).toHaveCount(0)
  await expect(retry).toHaveCount(0)
  expect(retriedIds.sort()).toEqual([...failedIds].sort())
  const saved = await savedTakes(page, phrase)
  expect(saved).toHaveLength(2)
  expect(saved).toEqual(
    expect.arrayContaining(
      [...failedIds].map((id) => expect.objectContaining({ id, complete: true })),
    ),
  )
})

test('a local write failure stays a capture error', async ({ page }) => {
  await page.evaluate(() => {
    const put = IDBObjectStore.prototype.put
    IDBObjectStore.prototype.put = function (value: unknown, key?: IDBValidKey) {
      if (
        String(key).startsWith('take-v2:') &&
        (value as { status?: string }).status === 'pending'
      ) {
        throw new DOMException('Browser recording storage is full', 'QuotaExceededError')
      }
      return put.call(this, value, key)
    }
  })
  const practiceCard = card(page)
  await practiceCard.getByRole('button', { name: 'Record a take' }).click()
  await practiceCard.getByRole('button', { name: 'Stop recording' }).click()
  await expect(practiceCard.getByRole('alert')).toHaveText(
    /QuotaExceededError.*Browser recording storage is full/,
  )
  await expect(practiceCard.getByRole('button', { name: 'Record a take' })).toBeEnabled()
  await expect(practiceCard.getByRole('button', { name: 'Retry save' })).toHaveCount(0)
  await expect(practiceCard.getByText('Audio remains on this device.')).toHaveCount(0)
})
