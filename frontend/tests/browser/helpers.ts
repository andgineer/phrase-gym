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

export const phraseSearch = (page: Page) =>
  page.getByRole('searchbox', { name: 'Search phrases or concepts' })

export const libraryPhrase = (page: Page, text: string) =>
  page.getByRole('list', { name: 'Phrases' }).getByRole('button', { name: text, exact: true })

export async function revealAnswer(page: Page, tap = false) {
  const show = page.getByRole('button', { name: 'Show', exact: true })
  const answer = page.getByTestId('practice-phrase')
  await expect(show.or(answer)).toBeVisible()
  if (await show.count()) await (tap ? show.tap() : show.click())
}

export async function rateNew(page: Page, tap = false, wait = true) {
  await revealAnswer(page, tap)
  const answer = page.getByTestId('practice-phrase')
  const again = page.getByRole('button', { name: 'Again', exact: true })
  await (tap ? again.tap() : again.click())
  if (wait) await expect(answer).toHaveCount(0)
}

// Midday keeps whole-day returns on the same learning day across a daylight-saving change.
export function middayTomorrow() {
  const instant = new Date(Date.now() + 24 * 3600 * 1000)
  instant.setUTCHours(11, 0, 0, 0)
  return instant.toISOString()
}
