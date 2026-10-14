import { expect } from '@playwright/test'
import { test } from './fixtures'

test('prepared variants rotate in later sessions and active sessions keep the pinned cue', async ({
  page,
  request,
}) => {
  await request.put('/api/v1/settings', {
    data: { native_language: 'ru', ui_language_mode: 'learning' },
  })
  expect((await request.post('/_test/prepared-greeting')).ok()).toBe(true)
  const lessons = await (await request.get('/api/v1/lessons?language=en')).json()
  const lesson = lessons.items.find((item: { title: string }) => item.title === 'Greetings')
  const start = await request.post('/api/v1/sessions', {
    data: { id: crypto.randomUUID(), language: 'en', lesson_id: lesson.id },
  })
  expect(start.ok()).toBe(true)
  const first = await start.json()
  expect(first.manifest.targets).toHaveLength(1)
  await page.goto(`/learn/en/practice/${first.id}`)
  await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  const before = await page.getByTestId('practice-phrase').textContent()
  expect(before).toContain('And yours?')
  await page.reload()
  await expect(page.getByTestId('practice-phrase')).toHaveText(before!)
  // A new session replaces the previous session explicitly; presentation history rotates masks.
  const next = await request.post('/api/v1/sessions', {
    data: {
      id: crypto.randomUUID(),
      language: 'en',
      lesson_id: lesson.id,
      takeover_session_id: first.id,
    },
  })
  expect(next.ok()).toBe(true)
  const second = await next.json()
  expect(second.manifest.targets[0].phrase.id).toBe(first.manifest.targets[0].phrase.id)
  expect(second.manifest.targets[0].cue_id).not.toBe(first.manifest.targets[0].cue_id)
  await page.goto(`/learn/en/practice/${second.id}`)
  await expect(page.getByTestId('practice-phrase')).toContainText('Not too bad')
  await expect(page.getByTestId('practice-phrase')).not.toContainText('And yours?')
  const retained = await (await request.get(`/api/v1/sessions/${first.id}`)).json()
  expect(retained.manifest).toEqual(first.manifest)
  expect(retained.status).toBe('superseded')
  const concept = await (
    await request.get('/api/v1/phrases/search?language=en&q=reciprocity')
  ).json()
  expect(concept.items).toHaveLength(1)
})
