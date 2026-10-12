import { readFileSync } from 'node:fs'
import { randomUUID } from 'node:crypto'
import { test, expect } from '@playwright/test'

for (const view of ['lookup', 'lesson', 'lessons']) {
  test(`${view} pagination serializes repeated clicks without hiding items`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning', last_learning_language: 'en' },
    })
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
      (
        await request.post(`/api/v1/imports/${staged.id}/publish`, { data: { reviewed: true } })
      ).ok(),
    ).toBe(true)
    const catalog = await (await request.get(`/api/v1/lessons?language=en&q=${token}`)).json()
    const path =
      view === 'lesson'
        ? `/learn/en/lessons/${catalog.items[0].id}`
        : `/learn/en?${view === 'lessons' ? 'lessonQ' : 'q'}=${token}`
    await page.goto(path)
    const section =
      view === 'lessons'
        ? page.locator('.session-entry')
        : view === 'lookup'
          ? page.locator('.lookup')
          : page.locator('.card')
    const more = section.getByRole('button', { name: 'Show more', exact: true })
    await expect(more).toBeEnabled()
    let release!: () => void
    const gate = new Promise<void>((resolve) => {
      release = resolve
    })
    let requests = 0
    await page.route('**/api/v1/**', async (route) => {
      if (new URL(route.request().url()).searchParams.get('offset') === '50') {
        requests += 1
        await gate
      }
      await route.continue()
    })
    await more.evaluate((button: HTMLButtonElement) => {
      button.click()
      button.click()
    })
    await expect.poll(() => requests).toBe(1)
    await expect(more).toBeDisabled()
    release()
    const rows =
      view === 'lessons'
        ? section.locator('option[value]:not([value=""])')
        : section.locator('.phrase-list a')
    await expect(rows).toHaveCount(100)
    if (view !== 'lessons') {
      await more.click()
      await expect(rows).toHaveCount(120)
    }
    await expect(more).toHaveCount(0)
    const identities = await rows.evaluateAll((elements) =>
      elements.map((element) => element.getAttribute('href') ?? element.getAttribute('value')),
    )
    expect(new Set(identities).size).toBe(view === 'lessons' ? 100 : 120)
    expect(requests).toBe(1)
  })
}
