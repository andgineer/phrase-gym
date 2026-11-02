import type { Locator, Page } from '@playwright/test'
import { expect } from './fixtures'

export const savedForCoach = 'Saved. The phrases appear after Coach prepares audio.'

export async function importStatus(page: Page, packageId: string) {
  return (await page.request.get(`/api/imports/${packageId}`)).json()
}

export async function awaitAdded(page: Page, packageId: string, added: number) {
  await expect
    .poll(() => importStatus(page, packageId))
    .toMatchObject({ status: 'added', added_count: added })
}

export async function importPackage(
  page: Page,
  value: { package_id: string },
  added: number,
  tap = false,
) {
  const press = (button: Locator) => (tap ? button.tap() : button.click())
  await press(page.getByRole('button', { name: 'Add material' }))
  await page.getByLabel('Phrase package JSON').fill(JSON.stringify(value))
  await press(page.getByRole('button', { name: 'Import', exact: true }))
  await expect(page.getByText(savedForCoach)).toBeVisible()
  await press(page.getByRole('button', { name: 'Close' }))
  await awaitAdded(page, value.package_id, added)
  await press(page.getByRole('navigation').getByRole('button', { name: 'Practice' }))
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
