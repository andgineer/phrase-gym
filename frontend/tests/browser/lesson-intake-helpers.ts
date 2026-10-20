import type { Page } from '@playwright/test'

export async function fillLesson(page: Page, _title = 'Inbox practice', send = true) {
  await page.getByLabel('Exact English phrase', { exact: true }).fill("Let's get started.")
  if (send) await sendPhrase(page)
}

export async function sendPhrase(page: Page) {
  await page.getByRole('button', { name: 'Send', exact: true }).click()
}
