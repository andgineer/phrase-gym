import { randomUUID } from 'node:crypto'
import type { Page } from '@playwright/test'
import type { components } from '../../src/api.generated'
import { expect } from './fixtures'

type Card = components['schemas']['PracticeCard']
type State = components['schemas']['PracticeState']
type Outcome = components['schemas']['CompletionRequest']['outcome']
type Settings = components['schemas']['Settings']

async function post<T>(page: Page, path: string, data: object): Promise<T> {
  const response = await page.request.post(`/api/practice/${path}`, { data })
  expect(response.status()).toBe(200)
  return response.json()
}

export async function setSettings(page: Page, changes: Partial<Settings>) {
  const settings = await (await page.request.get('/api/settings')).json()
  const response = await page.request.put('/api/settings', { data: { ...settings, ...changes } })
  expect(response.status()).toBe(200)
}

export const startDaily = (page: Page) => post<State>(page, 'daily', { lesson_id: null })

export async function completeCard(page: Page, card: Card, outcome: Outcome = 'not_yet') {
  const identity = { phrase_id: card.phrase_id, view_token: card.view_token }
  if (outcome !== 'skipped' && !card.revealed) await post<Card>(page, 'reveal', identity)
  const result = await post<components['schemas']['CompletionResult']>(page, 'complete', {
    ...identity,
    outcome,
    completion_id: randomUUID(),
  })
  expect(result.state).not.toBeNull()
  return result.state!
}

// Prerequisite history uses the real API; browser specs exercise only their target interaction.
export async function finishDaily(page: Page) {
  let state = await startDaily(page)
  const completed: Card[] = []
  while (state.card) {
    const revealed = await post<Card>(page, 'reveal', {
      phrase_id: state.card.phrase_id,
      view_token: state.card.view_token,
    })
    completed.push(revealed)
    state = await completeCard(page, revealed, revealed.kind === 'new' ? 'not_yet' : 'got_it')
  }
  return completed
}

export async function seedTake(page: Page, card: Card) {
  const response = await page.request.post('/api/recordings', {
    data: {
      id: randomUUID(),
      phrase_id: card.phrase_id,
      view_token: card.view_token,
      mode: card.mode,
      kind: card.kind,
      cue: card.cue,
      hint: card.hint,
      captured_at: new Date().toISOString(),
      mime_type: 'audio/webm;codecs=opus',
      duration_ms: 600,
      complete: true,
      audio_base64: 'AQIDBA==',
    },
  })
  expect(response.status()).toBe(200)
}
