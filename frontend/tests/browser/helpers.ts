import type { Page } from '@playwright/test'
import { expect } from './fixtures'

export async function importPackage(page: Page, value: object, added: string, tap = false) {
  const press = (name: string, exact = false) => {
    const button = page.getByRole('button', { name, exact })
    return tap ? button.tap() : button.click()
  }
  await press('Add material')
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(value))
  await press('Import', true)
  await expect(page.getByText(added)).toBeVisible()
  await press('Close')
}

export function playedMedia(page: Page) {
  return page.evaluate(() => (window as unknown as { __playedUrls?: string[] }).__playedUrls ?? [])
}

export async function phraseId(page: Page, text: string): Promise<string> {
  const found = await page.request.get(`/api/phrases?query=${encodeURIComponent(text)}`)
  return (await found.json()).items[0].id
}

export async function savedTakes(page: Page, text: string) {
  const id = await phraseId(page, text)
  return (await (await page.request.get(`/api/phrases/${id}/recordings?limit=100`)).json()).items
}
