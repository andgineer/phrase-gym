import type { Page } from '@playwright/test'
import { expect } from './fixtures'

export async function importStatus(page: Page, packageId: string) {
  return (await page.request.get(`/api/imports/${packageId}`)).json()
}

export function silentReference(): string {
  const samples = 2205
  const wav = Buffer.alloc(44 + samples * 2)
  wav.write('RIFF', 0)
  wav.writeUInt32LE(wav.length - 8, 4)
  wav.write('WAVEfmt ', 8)
  wav.writeUInt32LE(16, 16)
  wav.writeUInt16LE(1, 20)
  wav.writeUInt16LE(1, 22)
  wav.writeUInt32LE(22050, 24)
  wav.writeUInt32LE(44100, 28)
  wav.writeUInt16LE(2, 32)
  wav.writeUInt16LE(16, 34)
  wav.write('data', 36)
  wav.writeUInt32LE(samples * 2, 40)
  return wav.toString('base64')
}

export type ExternalPackage = { package_id: string; lessons: { phrases: { item_id: string }[] }[] }

// External agents submit complete packages with audio through the API; the browser has no import.
export async function importPackage(
  page: Page,
  value: ExternalPackage,
  added: number,
  tap = false,
) {
  const items = value.lessons.flatMap((lesson) => lesson.phrases)
  const references = Object.fromEntries(items.map((item) => [item.item_id, silentReference()]))
  const response = await page.request.post('/api/imports', {
    data: { package: value, references },
  })
  expect(await response.json()).toMatchObject({ status: 'added', added_count: added })
  const practice = page.getByRole('navigation').getByRole('button', { name: 'Practice' })
  await (tap ? practice.tap() : practice.click())
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
  if (!(await show.count())) return
  await (tap ? show.tap() : show.click())
  // The back moves the audio controls; a click aimed at the front could land on Ref instead.
  await expect(answer).toBeVisible()
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
