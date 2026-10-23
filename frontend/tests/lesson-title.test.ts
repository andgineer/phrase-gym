import { readFileSync } from 'node:fs'
import { expect, test } from 'vitest'
import { lessonTitle } from '../src/lessonTitle'

const examples: { items: string[]; title: string }[] = JSON.parse(
  readFileSync(new URL('../../fixtures/lesson-titles.json', import.meta.url), 'utf8'),
)

test('lessons are titled from their items exactly as Coach titles them', () => {
  for (const { items, title } of examples) {
    expect(lessonTitle(items)).toBe(title)
    expect([...lessonTitle(items)].length).toBeLessThanOrEqual(60)
  }
})
