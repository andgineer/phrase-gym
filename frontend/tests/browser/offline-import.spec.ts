import { expect, test } from './fixtures'

test.use({ coachEnabled: false })

test('offline request remains pending across reload while study stays available', async ({
  page,
}) => {
  const phrasePackage = {
    format: 'phrase-gym.package',
    version: 1,
    package_id: 'offline-browser-package',
    source_id: 'offline-browser-source',
    source_label: 'Offline course',
    learning_language: 'en',
    card_language: 'en',
    help_language: 'en',
    cards: [
      {
        item_id: 'first',
        answer: 'I can take care of that.',
        explanation: 'Offer to handle a task.',
        cues: [{ situation: 'A colleague asks who can handle the task.' }],
      },
    ],
  }
  await page.goto('/')
  await page.getByRole('button', { name: 'Add material' }).click()
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(phrasePackage))
  await page.getByRole('button', { name: 'Import' }).click()
  await expect(
    page.getByText('Pending Coach. You can keep studying while it is offline.'),
  ).toBeVisible()
  await page.getByRole('button', { name: 'Close' }).click()
  await page.reload()
  await expect(page.getByText('Pending Coach', { exact: true })).toBeVisible()
  await expect(page.getByText('Add your first phrase batch', { exact: true })).toBeVisible()
  const state = await page.evaluate(async () =>
    (await fetch('/api/imports/offline-browser-package')).json(),
  )
  expect(state.status).toBe('pending')
})
