import { expect } from '@playwright/test'
import { test } from './preview-server'
import { englishLesson } from './english-lesson'
import { syntheticCapture, retainedAudio } from './synthetic-capture'
import { expectComparison } from './native-playback'

test('Practice now retains scoped voluntary ratings and takes across reload and restart without changing the schedule', async ({
  page,
  request,
  preview,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  const { publication } = await englishLesson(request)
  await syntheticCapture(page)
  const lesson = `/learn/en/lessons/${publication.lesson_id}`
  await page.goto(lesson)
  await page.getByRole('button', { name: 'Start lesson drill', exact: true }).click()
  await page.getByRole('button', { name: 'Next', exact: true }).click()
  await expect(page.getByRole('button', { name: 'Practice now', exact: true })).toBeVisible()
  // Normal Start still supplies the first independent recall after genuine study.
  await page.goto(lesson)
  await page.getByRole('button', { name: 'Start lesson drill', exact: true }).click()
  await page.getByRole('button', { name: 'Show answer', exact: true }).click()
  await page.getByRole('button', { name: /Recalled$/ }).click()
  await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
  const duePath = `/api/v1/sessions/readiness?language=en&lesson_id=${publication.lesson_id}`
  const before = await (await request.get(duePath)).json()
  expect(before).toMatchObject({ due_count: 0, new_count: 0, practice_ready_count: 1 })
  // Completion launches another round with its exact lesson scope.
  await page.getByRole('button', { name: 'Practice now', exact: true }).click()
  await expect(
    page.getByText('Voluntary practice · ratings do not change your schedule.', { exact: true }),
  ).toBeVisible()
  const current = await (await request.get('/api/v1/sessions/current')).json()
  expect(current.manifest).toMatchObject({
    language: 'en',
    lesson_id: publication.lesson_id,
    voluntary: true,
  })
  await page.getByRole('button', { name: /Show & explain/ }).click()
  await page.getByRole('button', { name: 'Record', exact: true }).click()
  await expect.poll(() => retainedAudio(page)).toBeGreaterThan(0)
  await page.getByRole('button', { name: 'Stop recording', exact: true }).click()
  await expectComparison(page, 1)
  await page.getByRole('button', { name: /Not yet$/ }).click()
  await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
  const result = await (await request.get(`/api/v1/sessions/${current.id}`)).json()
  expect(result.attempts[0]).toMatchObject({
    rating: 'not_yet',
    eligibility_reason: 'voluntary_practice',
    original: { supported: true },
  })
  const receipts = await (await request.get(`/api/v1/sessions/${current.id}/events`)).json()
  expect(receipts.every((r: { schedule_changed: boolean }) => !r.schedule_changed)).toBe(true)
  expect(await (await request.get(duePath)).json()).toEqual(before)
  const takesPath = `/api/v1/phrases/${publication.target_ids[0]}/recordings`
  await expect.poll(async () => (await (await request.get(takesPath)).json()).length).toBe(1)
  const [take] = await (await request.get(takesPath)).json()
  const bytes = await (await request.get(`/api/v1/recordings/${take.id}`)).body()
  await preview.restart()
  await page.reload()
  await expect(page.getByRole('heading', { name: 'Session complete', exact: true })).toBeVisible()
  expect(await (await request.get(`/api/v1/recordings/${take.id}`)).body()).toEqual(bytes)
  expect(await (await request.get(`/api/v1/sessions/${current.id}`)).json()).toEqual(result)
  await page.goto(lesson)
  await expect(page.getByRole('button', { name: 'Start lesson drill', exact: true })).toBeDisabled()
  await page.getByRole('button', { name: 'Practice now', exact: true }).click()
  await page.getByRole('button', { name: 'Library', exact: true }).click()
  const open = await (await request.get('/api/v1/sessions/current')).json()
  await page.getByRole('button', { name: 'Practice now', exact: true }).click()
  await expect(page).toHaveURL(new RegExp(`/practice/${open.id}$`))
  expect((await (await request.get('/api/v1/sessions/current')).json()).id).toBe(open.id)
})

test('Practice now is absent with an explicit explanation when nothing is ready', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  await page.goto('/learn/en')
  await expect(page.getByRole('button', { name: 'Practice now', exact: true })).toHaveCount(0)
  await expect(page.getByText(/No practice-ready material in this scope/)).toBeVisible()
})
