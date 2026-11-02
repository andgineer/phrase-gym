import { expect, test } from './fixtures'
import { importStatus, savedForCoach } from './helpers'

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

test('imports wait for offline Coach; failures arrive with the facts and can be dismissed', async ({
  page,
}) => {
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
  const packageId = phrasePackage.package_id
  const statusReads: string[] = []
  page.on('request', (request) => {
    if (request.method() === 'GET' && new URL(request.url()).pathname.startsWith('/api/imports'))
      statusReads.push(request.url())
  })
  await page.clock.install()
  await page.goto('/')
  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(phrasePackage))
  const importButton = page.getByRole('button', { name: 'Import', exact: true })
  await importButton.click()
  await expect(page.getByText(savedForCoach)).toBeVisible()
  await page.clock.runFor(60_000)
  expect(await importStatus(page, packageId)).toMatchObject({ status: 'pending' })

  await page.request.post('/api/imports/coach/next')
  await page.request.post(`/api/imports/${packageId}/failure`, {
    data: { error: 'Voice unavailable' },
  })
  await page.reload()
  const notices = page.getByRole('list', { name: 'Import problems' }).getByRole('listitem')
  await expect(notices).toHaveCount(1)
  await expect(notices).toContainText(
    'Coach could not prepare Offline course (offline-browser-first): Voice unavailable',
  )
  await page.getByRole('button', { name: `Dismiss notice about ${packageId}` }).click()
  await expect(notices).toHaveCount(0)
  const facts = await (await page.request.get('/api/practice/facts')).json()
  expect(facts.notices).toEqual([])
  expect(await importStatus(page, packageId)).toMatchObject({
    status: 'error',
    error: 'Voice unavailable',
  })

  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(phrasePackage))
  await importButton.click()
  await expect(page.getByText(savedForCoach)).toBeVisible()
  expect(await importStatus(page, packageId)).toMatchObject({ status: 'pending', error: null })

  await page.request.post('/api/imports/coach/next')
  await page.request.post(`/api/imports/${packageId}/result`, {
    data: { result_id: packageId, references: { first: silentReference() } },
  })
  await page.clock.runFor(60_000)
  await expect(page.getByText(savedForCoach)).toBeVisible()
  await importButton.click()
  await expect(page.getByText('Added 1 phrases.')).toBeVisible()
  await page.getByRole('button', { name: 'Close' }).click()
  await expect(page.getByTestId('lesson-counts')).toHaveText('0 due · 1 new')
  expect(statusReads).toEqual([])
})
