import { expect, test } from './fixtures'

test.use({ coachEnabled: false })

function silentReference(): string {
  const samples = 2205
  const wav = Buffer.alloc(44 + samples * 2)
  wav.write('RIFF', 0)
  wav.writeUInt32LE(wav.length - 8, 4)
  wav.write('WAVEfmt ', 8)
  wav.writeUInt32LE(16, 16)
  wav.writeUInt16LE(1, 20)
  wav.writeUInt16LE(1, 22)
  wav.writeUInt32LE(22050, 24)
  wav.writeUInt32LE(44100, 28)
  wav.writeUInt16LE(2, 32)
  wav.writeUInt16LE(16, 34)
  wav.write('data', 36)
  wav.writeUInt32LE(samples * 2, 40)
  return wav.toString('base64')
}

test('two offline requests retain independent status and retry across reload', async ({ page }) => {
  const phrasePackage = {
    format: 'phrase-gym.package',
    version: 2,
    package_id: 'offline-browser-first',
    source_id: 'offline-browser-first-source',
    source_label: 'Offline course',
    learning_language: 'en',
    card_language: 'en',
    help_language: 'en',
    lessons: [
      {
        lesson_id: 'tasks',
        title: 'Tasks',
        phrases: [
          {
            item_id: 'first',
            text: 'I can take care of that.',
            explanation: 'Offer to handle a task.',
            cues: [{ situation: 'A colleague asks who can handle the task.' }],
            recall_hints: ['take care', 'handle'],
            usage_contexts: ['work tasks'],
          },
        ],
      },
    ],
  }
  const second = {
    ...phrasePackage,
    package_id: 'offline-browser-second',
    source_id: 'offline-browser-second-source',
  }
  const requestRow = (id: string) =>
    page.getByRole('list', { name: 'Import requests' }).locator('li').filter({ hasText: id })
  await page.goto('/')
  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(phrasePackage))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(requestRow(phrasePackage.package_id)).toContainText('Pending Coach')
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(second))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(requestRow(second.package_id)).toContainText('Pending Coach')
  await page.getByRole('button', { name: 'Close' }).click()
  await page.reload()
  await expect(page.getByText('Pending Coach', { exact: true })).toBeVisible()
  await expect(page.getByText('Add your first phrase batch', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Add material' }).click()
  await expect(requestRow(phrasePackage.package_id)).toContainText('Pending Coach')
  await expect(requestRow(second.package_id)).toContainText('Pending Coach')

  await page.evaluate(
    async ({ first, second, audio }) => {
      await fetch(`/api/imports/${first}/failure`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ error: 'Voice unavailable' }),
      })
      await fetch(`/api/imports/${second}/result`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ result_id: second, references: { first: audio } }),
      })
    },
    { first: phrasePackage.package_id, second: second.package_id, audio: silentReference() },
  )
  await expect(requestRow(phrasePackage.package_id).getByRole('alert')).toContainText(
    'Voice unavailable',
  )
  await expect(requestRow(second.package_id)).toContainText('Added 1 phrases.')
  await expect(page.getByText('Import needs attention', { exact: true })).toBeVisible()
  await page.reload()
  await expect(page.getByText('Import needs attention', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Add material' }).click()
  await expect(requestRow(phrasePackage.package_id).getByRole('alert')).toContainText(
    'Voice unavailable',
  )
  await expect(requestRow(second.package_id)).toContainText('Added 1 phrases.')
  await requestRow(phrasePackage.package_id)
    .getByRole('button', { name: 'Retry preparation' })
    .click()
  await expect(requestRow(phrasePackage.package_id)).toContainText('Pending Coach')
  await expect(requestRow(second.package_id)).toContainText('Added 1 phrases.')
})
