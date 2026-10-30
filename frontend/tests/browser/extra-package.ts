import type { Page } from '@playwright/test'
import { expect } from './fixtures'

const phrase = (item_id: string, text: string) => ({
  item_id,
  text,
  explanation: `Meaning of ${text}`,
  detailed_explanation: `Longer help for ${text}`,
  cues: [{ situation: `Situation for ${text}` }],
  recall_hints: [`${item_id} hint`, `${item_id} clue`],
  usage_contexts: ['small talk'],
})
export const texts = ['So far, so good.', 'Nice to meet you.', 'That works for me.']
export const extraPackage = {
  format: 'phrase-gym.package',
  version: 2,
  package_id: 'extra-browser-v1',
  source_id: 'extra-browser-source',
  source_label: 'Extra course',
  learning_language: 'en',
  card_language: 'en',
  help_language: 'en',
  lessons: [
    {
      lesson_id: 'small-talk',
      title: 'Small talk',
      phrases: texts.map((text, index) => phrase(`item-${index}`, text)),
    },
  ],
}
export const card = (page: Page) => page.locator('article.practice-card')
export const modeLine = (page: Page) => page.getByTestId('mode-line')
export const stopButton = (page: Page) => page.getByRole('button', { name: 'Stop', exact: true })
export const details = (page: Page) => page.locator('section.phrase-details')

export async function openDetails(page: Page, text: string, tap = false) {
  const button = page.getByRole('button', { name: `Open details: ${text}` })
  await (tap ? button.tap() : button.click())
  await expect(details(page).getByRole('heading', { name: text })).toBeVisible()
}
