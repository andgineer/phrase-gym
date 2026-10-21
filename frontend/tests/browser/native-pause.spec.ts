import { expect } from '@playwright/test'
import { test } from './preview-server'
import { englishLesson } from './english-lesson'
import { syntheticCapture, retainedAudio } from './synthetic-capture'

test('outside pause ends native playback quietly and leaves reference available', async ({
  page,
  request,
}) => {
  await englishLesson(request)
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.addInitScript(() => {
    const play = HTMLMediaElement.prototype.play
    HTMLMediaElement.prototype.play = function () {
      Object.assign(window, { __outsidePause: () => this.pause() })
      return play.call(this)
    }
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await page.getByRole('button', { name: /Reference$/ }).click()
  await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
  await page.evaluate(() => (window as unknown as { __outsidePause: () => void }).__outsidePause())
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  await expect(page.getByRole('button', { name: /Reference$/ })).toBeEnabled()
  await expect(page.getByRole('alert')).toHaveCount(0)
})

test('a delayed pause event from a previous source cannot stop native playback that is still running', async ({
  page,
  request,
}) => {
  await englishLesson(request)
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.addInitScript(() => {
    const play = HTMLMediaElement.prototype.play
    HTMLMediaElement.prototype.play = function () {
      return play.call(this).then(() => {
        this.dispatchEvent(new Event('pause'))
      })
    }
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await page.getByRole('button', { name: /Reference$/ }).click()
  await expect
    .poll(
      () =>
        page.evaluate(
          () =>
            (window as unknown as { __nativePlaybacks: { ended: boolean }[] }).__nativePlaybacks[0]
              ?.ended,
        ),
      { timeout: 15000 },
    )
    .toBe(true)
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  await expect(page.getByRole('alert')).toHaveCount(0)
})

test('outside pause ends learner comparison without starting reference and keeps both saved replays available', async ({
  page,
  request,
}) => {
  const { publication } = await englishLesson(request)
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await syntheticCapture(page)
  await page.addInitScript(() => {
    const play = HTMLMediaElement.prototype.play
    HTMLMediaElement.prototype.play = function () {
      Object.assign(window, { __outsidePause: () => this.pause() })
      return play.call(this)
    }
  })
  await page.goto('/learn/en')
  await page.getByRole('button', { name: 'Start session', exact: true }).click()
  await expect(page.getByTestId('practice-phrase')).toBeVisible()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Stop playback', exact: true })).toBeVisible()
  await page.evaluate(() => (window as unknown as { __outsidePause: () => void }).__outsidePause())
  await expect(page.getByRole('button', { name: 'Record', exact: true })).toBeEnabled()
  await expect(page.getByRole('button', { name: /My recording$/ })).toBeEnabled()
  await expect(page.getByRole('button', { name: /Reference$/ })).toBeEnabled()
  await expect(page.getByRole('alert')).toHaveCount(0)
  expect(
    await page.evaluate(
      () => (window as unknown as { __nativePlaybacks: { source: string }[] }).__nativePlaybacks,
    ),
  ).toMatchObject([{ source: expect.stringMatching(/^blob:/) }])
  const rows = await (
    await request.get(`/api/v1/phrases/${publication.target_ids[0]}/recordings`)
  ).json()
  expect(rows).toHaveLength(1)
})
