import { readFileSync } from 'node:fs'
import { randomUUID } from 'node:crypto'
import { expect } from '@playwright/test'
import type { APIRequestContext } from '@playwright/test'

export async function publishPagedLibrary(request: APIRequestContext) {
  const token = randomUUID()
  const fixture = JSON.parse(readFileSync('../fixtures/manual-en.json', 'utf8'))
  const targets = Array.from({ length: 120 }, (_, index) => ({
    ...fixture.targets[0],
    source_id: `target-${index}`,
    concepts: [token],
  }))
  const lessons = Array.from({ length: 100 }, (_, index) => ({
    source_id: `lesson-${index}`,
    title: `${token} ${String(index).padStart(3, '0')}`,
    target_source_ids: index === 0 ? targets.map((t) => t.source_id) : [targets[index].source_id],
  }))
  const staged = await (
    await request.post('/api/v1/imports', {
      data: { ...fixture, namespace: token, targets, lessons, dialogues: [] },
    })
  ).json()
  expect(staged.status).toBe('staged')
  expect(
    (await request.post(`/api/v1/imports/${staged.id}/publish`, { data: { reviewed: true } })).ok(),
  ).toBe(true)
  const catalog = await (await request.get(`/api/v1/lessons?language=en&q=${token}`)).json()
  return { token, lessonId: catalog.items[0].id as string }
}
