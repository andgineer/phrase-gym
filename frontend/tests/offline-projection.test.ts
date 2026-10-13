import { expect, it } from 'vitest'
import { localAttemptId, projectEvent } from '../src/offlineProjection'
import type { PracticeInput, PracticeSession } from '../src/practiceTypes'
const session = {
  id: '00000000-0000-0000-0000-000000000001',
  status: 'active',
  version: 1,
  cursor: 0,
  last_sequence: 0,
  queue: [{ target_index: 0, repair: false }],
  attempts: [],
  manifest: {
    targets: [{ phrase: { id: 'target', revision: 1 }, cue_id: 'cue', review_group_id: 'group' }],
  },
} as unknown as PracticeSession
function event(kind: PracticeInput['kind'], sequence: number): PracticeInput {
  return {
    id: String(sequence),
    kind,
    step: 0,
    sequence,
    occurred_at: '2026-10-13T00:00:00Z',
    modality: 'self_report',
    defer_repairs: kind === 'rate',
    ...(kind === 'rate' ? { rating: 'recalled', defer_repairs: true } : {}),
  }
}
it('uses the same UUID v5 attempt binding as the server', async () => {
  expect(await localAttemptId(session.id, 0)).toBe('6464bd92-0848-551c-b288-f009a204f509')
})
it('keeps original support, projects completion, and does not invent a schedule receipt', () => {
  let next = projectEvent(session, event('present', 1), 'attempt')
  next = projectEvent(next, event('explain', 2), 'attempt')
  next = projectEvent(next, event('respond', 3), 'attempt')
  next = projectEvent(next, event('capture_complete', 4), 'attempt')
  next = projectEvent(next, event('rate', 5), 'attempt')
  expect(next.status).toBe('completed')
  expect(next.attempts[0]?.original?.supported).toBe(true)
  expect(next.attempts[0]?.eligibility_reason).toBe('pending_receipt')
  expect(session.attempts).toHaveLength(0)
  expect(projectEvent(next, event('rate', 5), 'attempt')).toEqual(next)
})
it('help after the first response does not relabel recall', () => {
  let next = projectEvent(session, event('present', 1), 'attempt')
  next = projectEvent(next, event('answer', 2), 'attempt')
  next = projectEvent(next, event('respond', 3), 'attempt')
  expect(next.attempts[0]?.original?.supported).toBe(false)
})
it.each([
  [0, true],
  [300000, true],
  [300001, false],
  [-1, false],
])('uses checkpoint script exposure at elapsed %i milliseconds', (elapsed, supported) => {
  const state = { ...session, support_exposures: { target: ['2026-10-13T00:00:00Z'] } }
  const time = new Date(Date.parse(state.support_exposures.target[0]!) + elapsed).toISOString()
  let next = projectEvent(state, { ...event('present', 1), occurred_at: time }, 'attempt')
  next = projectEvent(next, { ...event('rate', 2), occurred_at: time }, 'attempt')
  expect(next.attempts[0]?.original?.supported).toBe(supported)
})
it('retains an offline script exposure and leaves the original response immutable', () => {
  let next = projectEvent(session, event('dialogue_exposure', 1), 'attempt')
  next = projectEvent(next, event('dialogue_begin', 2), 'attempt')
  next = projectEvent(next, event('present', 3), 'attempt')
  next = projectEvent(next, event('respond', 4), 'attempt')
  expect(next.dialogue_exposures).toEqual(['2026-10-13T00:00:00Z'])
  next = projectEvent(next, { ...event('rate', 5), occurred_at: '2026-10-13T00:06:00Z' }, 'attempt')
  expect(next.attempts[0]?.original?.supported).toBe(true)
})
