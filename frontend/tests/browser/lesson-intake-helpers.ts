import type { Page } from '@playwright/test'
import { tone } from './seed-library'

export async function fillLesson(page: Page) {
  await page.getByLabel('Lesson title', { exact: true }).fill('Opening a meeting')
  await page.getByLabel('Exact English phrase', { exact: true }).fill("Let's get started.")
  await page
    .getByLabel('Situation or meaning', { exact: true })
    .fill('All participants are present.')
  await page.getByLabel('Reference performer or source speaker').fill('Synthetic')
  await page.getByLabel(/Original reference audio/).setInputFiles({
    name: 'reference.wav',
    mimeType: 'audio/wav',
    buffer: Buffer.from(tone(), 'base64'),
  })
}
