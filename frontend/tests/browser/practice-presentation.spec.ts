import { randomUUID } from 'node:crypto'
import { expect } from '@playwright/test'
import { test } from './fixtures'
import type { components } from '../../src/api.generated'

for (const fullyHidden of [true, false]) {
  test(`${fullyHidden ? 'situation-only' : 'partial'} cue shows one instruction and meaningful phrase content`, async ({
    page,
    request,
  }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    const phrases: components['schemas']['PhrasePage'] = await (
      await request.get('/api/v1/phrases/search?language=en&q=greetings')
    ).json()
    const target = phrases.items[0].content
    const cue = target.cues![0]
    const visibleText = target.chunks
      .filter((chunk) => cue.visible_chunks.includes(chunk.id))
      .map((chunk) => chunk.text)
      .join('')
    target.purpose = fullyHidden ? cue.prompt : 'Different library purpose'
    if (fullyHidden) cue.visible_chunks = []
    const staged = await request.post('/api/v1/imports', {
      data: {
        namespace: 'browser.practice.presentation',
        language: 'en',
        source_revision: '1',
        targets: [target],
        lessons: [
          {
            source_id: 'presentation',
            title: 'Practice presentation',
            target_source_ids: [target.source_id],
          },
        ],
      },
    })
    expect(staged.ok()).toBe(true)
    const published = await request.post(`/api/v1/imports/${(await staged.json()).id}/publish`, {
      data: { reviewed: true },
    })
    expect(published.ok()).toBe(true)
    const lessons = await (
      await request.get('/api/v1/lessons?language=en&q=Practice%20presentation')
    ).json()
    const started = await request.post('/api/v1/sessions', {
      data: { id: randomUUID(), language: 'en', lesson_id: lessons.items[0].id },
    })
    expect(started.ok()).toBe(true)
    await page.goto(`/learn/en/practice/${(await started.json()).id}`)
    const card = page.locator('.practice-card')
    await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
    await expect(card.getByText(cue.prompt, { exact: true })).toHaveCount(1)
    if (fullyHidden) {
      await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
      await expect(card.locator('.gap')).toHaveCount(0)
      await page.reload()
      await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
      await expect(page.getByTestId('practice-phrase')).toHaveCount(0)
      await expect(card.getByText(cue.prompt, { exact: true })).toHaveCount(1)
    } else {
      await expect(page.getByTestId('practice-phrase')).toContainText(visibleText)
      await expect(card.locator('.gap')).toHaveCount(1)
      await expect(card.getByText(target.purpose, { exact: true })).toHaveCount(0)
    }
    await page.getByRole('button', { name: 'Show answer', exact: true }).click()
    await expect(page.getByTestId('practice-phrase')).toHaveText(target.text)
    await expect(card.locator('.gap')).toHaveCount(0)
    await expect(card.getByText(cue.prompt, { exact: true })).toHaveCount(1)
    await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toHaveCount(0)
    await expect(page.getByRole('button', { name: 'Show & explain' })).toBeEnabled()
    await page.reload()
    await expect(page.getByTestId('practice-phrase')).toHaveText(target.text)
    await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toHaveCount(0)
  })
}

for (const action of ['Show & explain', 'Reference']) {
  test(`${action} reveals the answer and hides its redundant action`, async ({ page, request }) => {
    await request.put('/api/v1/settings', {
      data: { native_language: 'ru', ui_language_mode: 'learning' },
    })
    await page.goto('/learn/en')
    await page.getByRole('button', { name: 'Start session', exact: true }).click()
    await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toBeEnabled()
    await page.getByRole('button', { name: action }).click()
    await expect(page.getByTestId('practice-phrase')).not.toContainText('_____')
    await expect(page.getByRole('button', { name: 'Show answer', exact: true })).toHaveCount(0)
    await expect(page.getByRole('button', { name: 'Recalled' })).toBeEnabled()
  })
}
