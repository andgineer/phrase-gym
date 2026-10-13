import { expect } from '@playwright/test'
import { test } from './fixtures'
import type { components } from '../../src/api.generated'
import type { Language } from '../../src/api'

type Explanation = components['schemas']['Explanation']
const meaning = 'Как проходит твой день?'
const cases: {
  name: string
  explanations: Partial<Record<Language, Explanation>>
  available: boolean
}[] = [
  {
    name: 'another language only',
    explanations: { en: { meaning: 'Ask about the day so far.' } },
    available: false,
  },
  {
    name: 'blank native content',
    explanations: { ru: { meaning: ' ', usage: ' ', breakdown: [] } },
    available: false,
  },
  { name: 'translation only', explanations: { ru: { meaning } }, available: true },
  {
    name: 'extended explanation',
    explanations: {
      ru: {
        meaning,
        usage: 'Вежливый вопрос о том, как проходит день.',
        breakdown: ['so far — до настоящего момента'],
      },
    },
    available: true,
  },
]

for (const example of cases) {
  test(`${example.name} offers native help only when it has content`, async ({ page, request }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    const phrases: components['schemas']['PhrasePage'] = await (
      await request.get('/api/v1/phrases/search?language=en&q=greetings')
    ).json()
    const target = { ...phrases.items[0].content, explanations: example.explanations }
    const staged = await request.post('/api/v1/imports', {
      data: {
        namespace: 'browser.native-help',
        language: 'en',
        source_revision: '1',
        targets: [target],
        lessons: [
          { source_id: 'help', title: 'Native help', target_source_ids: [target.source_id] },
        ],
      },
    })
    expect(staged.ok()).toBe(true)
    const report = await staged.json()
    expect(report.status).toBe('staged')
    expect(
      (
        await request.post(`/api/v1/imports/${report.id}/publish`, { data: { reviewed: true } })
      ).ok(),
    ).toBe(true)
    const lessons = await (await request.get('/api/v1/lessons?language=en&q=Native%20help')).json()
    await page.goto(`/learn/en/lessons/${lessons.items[0].id}`)
    await page.getByRole('button', { name: 'Start lesson drill', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
    const session = await (await request.get('/api/v1/sessions/current')).json()
    const phraseId = session.manifest.targets[0].phrase.id
    const explain = page.getByRole('button', { name: 'Show & explain' })
    if (example.available) {
      await expect(explain).toBeEnabled()
      await explain.click()
      await expect(page.locator('.practice-card .explanation')).toContainText(meaning)
      await expect(page.locator('.practice-card .explanation')).toHaveAttribute('lang', 'ru')
      if (example.name === 'translation only')
        await expect(page.locator('.practice-card .explanation p')).toHaveCount(1)
    } else {
      await expect(explain).toHaveCount(0)
      await page.getByRole('button', { name: 'Show answer', exact: true }).click()
      await expect(page.getByTestId('practice-phrase')).toHaveText(target.text)
      await expect(page.locator('.practice-card .explanation')).toHaveCount(0)
      await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    }
    await page.reload()
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
    await expect(explain).toHaveCount(example.available ? 1 : 0)
    await expect(page.locator('.practice-card')).not.toContainText('not prepared yet')

    let exposureKind = ''
    await page.route('**/api/v1/exposures', async (route) => {
      exposureKind = route.request().postDataJSON().kind
      await route.continue()
    })
    await page.goto(`/learn/en/phrases/${phraseId}`)
    await expect(page.locator('.phrase-card')).toBeVisible()
    await expect(page.locator('.phrase-card .explanation')).toHaveCount(example.available ? 1 : 0)
    await expect.poll(() => exposureKind).toBe(example.available ? 'explanation' : 'answer')
    await expect(page.locator('.phrase-card')).not.toContainText('not been prepared yet')
    if (example.name === 'translation only') {
      await expect(page.locator('.phrase-card .explanation')).toContainText(meaning)
      await expect(page.getByRole('heading', { name: 'Usage', exact: true })).toHaveCount(0)
    }
    if (example.name === 'another language only') {
      await page.getByRole('button', { name: 'Resume session', exact: true }).click()
      await page.getByRole('button', { name: 'Settings', exact: true }).click()
      await page.getByLabel('Your native language', { exact: true }).selectOption('en')
      await page.getByRole('button', { name: 'Save', exact: true }).click()
      await expect(explain).toBeEnabled()
      await explain.click()
      await expect(page.locator('.practice-card .explanation')).toContainText(
        'Ask about the day so far.',
      )
      await page.getByRole('button', { name: 'Settings', exact: true }).click()
      await page.getByLabel('Your native language', { exact: true }).selectOption('de')
      await page.getByRole('button', { name: 'Save', exact: true }).click()
      await expect(explain).toHaveCount(0)
      await expect(page.locator('.practice-card .explanation')).toHaveCount(0)
    }
  })
}
