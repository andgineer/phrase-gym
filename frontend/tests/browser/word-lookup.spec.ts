import { expect } from '@playwright/test'
import { readFile } from 'node:fs/promises'
import { test } from './preview-server'
import { prepareWithNativeFakeCoach } from './native-coach'

test('a word is listed by its word, its page leads with the word and native details wait for Show & explain', async ({
  page,
  request,
  preview,
}) => {
  const fixture = JSON.parse(await readFile('../fixtures/coach/working-set.json', 'utf8'))
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Add phrase', exact: true }).click()
  await page.getByRole('textbox', { name: 'Exact English phrase' }).fill(fixture.input)
  await page.getByRole('button', { name: 'Send', exact: true }).click()
  await expect(page.getByText(/Accepted — waiting for Coach/)).toBeVisible()
  const publication = await prepareWithNativeFakeCoach(preview.url)
  await page.goto('/learn/en')
  const title = "Let's get started. · Could you repeat that, please? · …"
  await expect(page.getByRole('option', { name: title, exact: true })).toHaveCount(1)
  const sense = 'Make something clearer or easier to understand.'
  const word = page.getByRole('link', { name: `clarify — ${sense}` })
  await expect(word).toBeVisible()
  await expect(page.getByRole('link', { name: /^Please clarify the deadline/ })).toHaveCount(0)
  await word.click()
  await expect(page.getByRole('heading', { level: 1, name: 'clarify', exact: true })).toBeVisible()
  await expect(page.locator('.word-sense')).toHaveText(sense)
  await expect(page.locator('.phrase-card')).not.toContainText('_____')
  const examples = page.locator('.examples li')
  await expect(examples).toHaveCount(3)
  await expect(examples.nth(0)).toContainText('Please clarify the deadline.')
  await expect(examples.nth(2)).toContainText('We need to clarify the plan.')
  await expect(
    page.getByRole('button', { name: 'Reference: Can you clarify this point?', exact: true }),
  ).toBeEnabled()
  await expect(page.locator('.explanation')).toContainText(sense)
  await expect(page.locator('.phrase-details')).toHaveCount(0)
  await page.getByRole('button', { name: /Show & explain/ }).click()
  const details = page.locator('.phrase-details')
  await expect(details).toHaveAttribute('lang', 'ru')
  await expect(details.locator('.details-word')).toContainText('clarify')
  await expect(details.locator('li')).toHaveCount(3)
  await expect(details.getByRole('heading', { name: 'Формы' })).toBeVisible()
  await page.getByRole('button', { name: /Show & explain/ }).click()
  await expect(page.locator('.phrase-details')).toHaveCount(0)
  await page.goto(`/learn/en/lessons/${publication.lesson_id}`)
  await expect(page.getByRole('heading', { level: 1, name: title, exact: true })).toBeVisible()
  await expect(page.getByRole('link', { name: `clarify — ${sense}` })).toBeVisible()
  await expect(
    page.getByRole('link', { name: /^Could we move the meeting to Friday\?/ }),
  ).toBeVisible()
})
