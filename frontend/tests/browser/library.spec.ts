import type { Page } from '@playwright/test'
import { installFakeMedia } from './fake-media'
import { expect, test } from './fixtures'
import { importPackage as importWith } from './helpers'

test.use({ viewport: { width: 390, height: 844 }, hasTouch: true, isMobile: true })

const phrase = (item_id: string, text: string, usage_contexts: string[], situation: string) => ({
  item_id,
  text,
  explanation: `Meaning of ${text}`,
  detailed_explanation: `Longer help for ${text}`,
  cues: [{ situation }],
  recall_hints: [`${item_id} hint`, `${item_id} clue`],
  usage_contexts,
})

function organizedPackage(packageId: string) {
  return {
    format: 'phrase-gym.package',
    version: 2,
    package_id: packageId,
    source_id: `${packageId}-source`,
    source_label: 'Mobile course',
    source_link: 'https://example.test/mobile',
    learning_language: 'en',
    card_language: 'en',
    help_language: 'en',
    lessons: [
      {
        lesson_id: 'small-talk',
        title: 'Small talk',
        phrases: [
          phrase(
            'so-far',
            'So far, so good.',
            ['small talk', 'progress update'],
            'Asked: How is it going?',
          ),
          phrase('nice', 'Nice to meet you.', ['greetings'], 'You are introduced to someone.'),
        ],
      },
      {
        lesson_id: 'meetings',
        title: 'Meetings',
        phrases: [
          phrase(
            'walk',
            'Could you walk me through that?',
            ['meetings'],
            'A process was too fast.',
          ),
          phrase('works', 'That works for me.', ['scheduling'], 'Accept the proposed time.'),
        ],
      },
    ],
  }
}

const importPackage = (page: Page, value: object, added: string) =>
  importWith(page, value, added, true)

async function noHorizontalScroll(page: Page) {
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390)
}

test.beforeEach(async ({ page }) => installFakeMedia(page))

test('mobile import, lesson choice, search, details and confirmed Delete', async ({ page }) => {
  await page.goto('/')
  await importPackage(page, organizedPackage('mobile-library-v1'), 'Added 4 phrases.')
  const counts = page.getByTestId('lesson-counts')
  await expect(counts).toHaveText('0 due · 4 new')
  await page.getByRole('button', { name: 'Daily', exact: true }).tap()
  const card = page.locator('article.practice-card')
  await expect(card.getByRole('heading', { name: 'So far, so good.' })).toBeVisible()
  await card.getByRole('button', { name: 'Record a take' }).tap()
  await card.getByRole('button', { name: 'Stop recording' }).tap()
  await expect(card.getByRole('button', { name: '▶ Mine' })).toBeVisible()
  await expect(page.getByText(/take\(s\) waiting to save/)).toHaveCount(0)

  await page.getByRole('button', { name: 'Practice', exact: true }).tap()
  const lessonChoice = page.getByLabel('Lesson to drill')
  await expect(counts).toHaveText('0 due · 4 new')
  await page.getByLabel('Search lessons').fill('meet')
  await expect(lessonChoice.locator('option')).toHaveText([
    'All lessons',
    'Meetings · Mobile course',
  ])
  await lessonChoice.selectOption({ label: 'Meetings · Mobile course' })
  await expect(counts).toHaveText('0 due · 2 new')
  const results = page.getByRole('list', { name: 'Phrases' })
  await expect(results.locator('strong')).toHaveText([
    'Could you walk me through that?',
    'That works for me.',
  ])
  await noHorizontalScroll(page)

  const find = page.getByLabel('Find a phrase')
  await find.fill('progress')
  await expect(page.getByText('Searching all lessons.')).toBeVisible()
  await expect(results.locator('strong')).toHaveText(['So far, so good.'])
  await expect(results).toContainText('Used for: small talk · progress update')
  await find.fill('zzz')
  await expect(page.getByText('No matches', { exact: true })).toBeVisible()

  await find.fill('how is it going')
  await page.getByRole('button', { name: 'Open details: So far, so good.' }).tap()
  const details = page.locator('.phrase-details')
  await expect(details.getByRole('heading', { name: 'So far, so good.' })).toBeVisible()
  await expect(details).toContainText('Used for: small talk · progress update')
  await expect(details).toContainText('Lesson: Small talk · Source: Mobile course')
  await expect(details).toContainText('New: not studied yet')
  await expect(details.getByText('Longer help for So far, so good.')).toHaveCount(0)
  await details.getByRole('button', { name: 'Details', exact: true }).tap()
  await expect(details.getByText('Longer help for So far, so good.')).toBeVisible()
  await expect(details.getByText(/ · Daily · First meeting$/)).toBeVisible()
  await expect(details.getByRole('button', { name: '▶ Mine' })).toHaveCount(1)
  await details.getByRole('button', { name: '▶ Phrase' }).tap()
  await expect
    .poll(() =>
      page.evaluate(() =>
        ((window as unknown as { __playedMedia?: HTMLMediaElement[] }).__playedMedia ?? []).map(
          (media) => new URL(media.src).pathname,
        ),
      ),
    )
    .toContainEqual(expect.stringMatching(/^\/api\/phrases\/.+\/reference$/))
  await noHorizontalScroll(page)

  const phraseId = await page.evaluate(
    async () => (await (await fetch('/api/phrases?query=so%20far')).json()).items[0].id as string,
  )
  await details.getByRole('button', { name: 'Delete', exact: true }).tap()
  const confirm = page.getByRole('dialog', { name: 'Delete “So far, so good.”?' })
  await expect(confirm).toContainText('its saved recordings will be permanently removed')
  await confirm.getByRole('button', { name: 'Cancel' }).tap()
  await expect(confirm).toBeHidden()
  await expect(details.getByRole('heading', { name: 'So far, so good.' })).toBeVisible()
  expect((await page.request.get(`/api/phrases/${phraseId}`)).status()).toBe(200)
  await details.getByRole('button', { name: 'Delete', exact: true }).tap()
  await confirm.getByRole('button', { name: 'Delete permanently' }).tap()
  await expect(page.getByText('Deleted “So far, so good.”.')).toBeVisible()
  expect((await page.request.get(`/api/phrases/${phraseId}`)).status()).toBe(404)
  await find.fill('so far')
  await expect(page.getByText('No matches', { exact: true })).toBeVisible()

  await find.fill('')
  await page.getByLabel('Search lessons').fill('')
  await lessonChoice.selectOption({ label: 'Small talk · Mobile course' })
  await expect(results.locator('strong')).toHaveText(['Nice to meet you.'])
  await expect(counts).toHaveText('0 due · 1 new')
  await page.getByLabel('Search lessons').fill('meet')
  await expect(counts).toHaveCount(0)
  await page.getByLabel('Search lessons').fill('')
  await expect(counts).toHaveText('0 due · 1 new')
  await page.getByRole('button', { name: 'Delete: Nice to meet you.' }).tap()
  const listConfirm = page.getByRole('dialog', { name: 'Delete “Nice to meet you.”?' })
  await listConfirm.getByRole('button', { name: 'Cancel' }).tap()
  await expect(results.locator('strong')).toHaveText(['Nice to meet you.'])
  await page.getByRole('button', { name: 'Delete: Nice to meet you.' }).tap()
  await listConfirm.getByRole('button', { name: 'Delete permanently' }).tap()
  await expect(
    page.getByRole('status').filter({ hasText: 'No phrases in this lesson' }),
  ).toBeVisible()
  await expect(counts).toHaveText('No phrases in this lesson')
  await page.getByRole('button', { name: 'Daily', exact: true }).tap()
  await expect(page.getByText('No phrases in this lesson.', { exact: true })).toBeVisible()

  await lessonChoice.selectOption({ label: 'All lessons' })
  await page.getByRole('button', { name: 'Daily', exact: true }).tap()
  await expect(card.getByRole('heading', { name: 'Could you walk me through that?' })).toBeVisible()
  await expect(page.getByRole('alert')).toHaveCount(0)
})

test('a take pending for a deleted phrase stays recoverable until discarded', async ({ page }) => {
  await page.goto('/')
  const single = organizedPackage('mobile-pending-v1')
  single.lessons = [{ ...single.lessons[1], phrases: [single.lessons[1].phrases[1]] }]
  await importPackage(page, single, 'Added 1 phrases.')
  await page.getByRole('button', { name: 'Daily', exact: true }).tap()
  await page.route('**/api/recordings', (route) =>
    route.request().method() === 'POST'
      ? route.fulfill({ status: 503, json: { detail: 'Temporary save failure' } })
      : route.continue(),
  )
  const card = page.locator('article.practice-card')
  await card.getByRole('button', { name: 'Record a take' }).tap()
  await card.getByRole('button', { name: 'Stop recording' }).tap()
  await expect(page.getByText('1 take(s) waiting to save.')).toBeVisible()

  await page.getByRole('button', { name: 'Practice', exact: true }).tap()
  await page.getByRole('button', { name: 'Delete: That works for me.' }).tap()
  await page.getByRole('button', { name: 'Delete permanently' }).tap()
  await expect(page.getByText('Deleted “That works for me.”.')).toBeVisible()
  await page.unroute('**/api/recordings')

  await page.getByRole('button', { name: 'Practice', exact: true }).tap()
  const refused = page.getByRole('alert').filter({ hasText: 'phrase was deleted' })
  await expect(refused).toContainText('A recording kept on this device cannot be saved')
  await page.reload()
  await expect(refused).toBeVisible()
  await refused.getByRole('button', { name: 'Discard recording' }).tap()
  await expect(refused).toHaveCount(0)
  expect(
    await page.evaluate(async () => {
      const db = await new Promise<IDBDatabase>((resolve, reject) => {
        const request = indexedDB.open('phrase-gym.practice.v1', 1)
        request.onsuccess = () => resolve(request.result)
        request.onerror = () => reject(request.error)
      })
      const keys = await new Promise<IDBValidKey[]>((resolve) => {
        const request = db.transaction('journal').objectStore('journal').getAllKeys()
        request.onsuccess = () => resolve(request.result)
      })
      db.close()
      return keys.filter((key) => String(key).startsWith('take-v2:'))
    }),
  ).toEqual([])
})

test('outstanding file cleanup stays visible until retry succeeds', async ({ page }) => {
  await page.route('**/api/cleanup', (route) =>
    route.fulfill({
      json: {
        pending: 1,
        error:
          '1 file(s) of deleted phrases could not be removed yet: permission denied. ' +
          'Fix the problem, then retry cleanup.',
      },
    }),
  )
  await page.goto('/')
  const notice = page.getByRole('alert').filter({ hasText: 'could not be removed yet' })
  await expect(notice).toContainText('permission denied')
  await page.unroute('**/api/cleanup')
  await page.getByRole('button', { name: 'Retry cleanup' }).tap()
  await expect(notice).toHaveCount(0)
  await expect(page.getByRole('button', { name: 'Retry cleanup' })).toHaveCount(0)
})
