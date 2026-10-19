import { expect } from '@playwright/test'
import { test } from './preview-server'
import { englishLesson } from './english-lesson'
import { retainedAudio, syntheticCapture } from './synthetic-capture'
import { expectComparison } from './native-playback'

test('published reference rehearsal and saved take survive reload and Server restart without Coach', async ({
  page,
  request,
  preview,
}) => {
  const { publication, asset } = await englishLesson(request)
  const settings = await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  expect(settings.ok()).toBe(true)
  const recordings: { exposure_id?: string; attempt_id?: string | null }[] = []
  page.on('request', (request) => {
    if (request.method() === 'POST' && request.url().endsWith('/api/v1/recordings'))
      recordings.push(request.postDataJSON())
  })
  await syntheticCapture(page)
  const target = publication.target_ids[0]
  await page.goto(`/learn/en/lessons/${publication.lesson_id}`)
  await page.getByRole('link', { name: /Let's get started/ }).click()
  await expect(page.getByRole('heading', { name: "Let's get started." })).toBeVisible()
  await expect(page.locator('.phrase-card')).toContainText(
    'Everyone has arrived. Open the meeting.',
  )
  await expect(page.locator('.explanation')).toContainText('Begin a shared activity.')
  const phrasePath = `/api/v1/phrases/${target}?language=en`
  const before = await (await request.get(phrasePath)).json()
  const referenceBytes = await (await request.get(`/api/v1/media/${asset.id}`)).body()
  await page.getByRole('button', { name: 'Reference', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Stop playback', exact: true }).click()
  await page.evaluate(() => {
    ;(window as unknown as { __nativePlaybacks: unknown[] }).__nativePlaybacks.length = 0
  })
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expectComparison(page, 1)
  const takesPath = `/api/v1/phrases/${target}/recordings`
  await expect.poll(async () => (await (await request.get(takesPath)).json()).length).toBe(1)
  const [take] = await (await request.get(takesPath)).json()
  const bytes = await (await request.get(`/api/v1/recordings/${take.id}`)).body()
  expect(bytes.length).toBeGreaterThan(0)
  expect(recordings).toHaveLength(1)
  expect(recordings[0].exposure_id).toBeTruthy()
  expect(recordings[0].attempt_id ?? null).toBeNull()
  for (const restart of [false, true]) {
    if (restart) await preview.restart()
    await page.reload()
    await expect(page.getByRole('heading', { name: "Let's get started." })).toBeVisible()
    await expect(page.getByRole('button', { name: /My recording$/ })).toBeEnabled()
    expect(await (await request.get(`/api/v1/recordings/${take.id}`)).body()).toEqual(bytes)
    expect(await (await request.get(`/api/v1/media/${asset.id}`)).body()).toEqual(referenceBytes)
    expect(await (await request.get(phrasePath)).json()).toEqual(before)
    await page.getByRole('button', { name: /My recording$/ }).click()
    await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
    await page.getByRole('button', { name: 'Stop playback', exact: true }).click()
  }
  const current = await request.get('/api/v1/sessions/current')
  expect(await current.json()).toBeNull()
})
