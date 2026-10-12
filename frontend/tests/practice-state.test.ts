import { expect, it } from 'vitest'
import { sessionCard } from '../src/practiceState'
import type { PracticeInput, PracticeSession } from '../src/practiceTypes'

it('reconstructs pending help without inventing a response or expanding audio help', () => {
  const session = { cursor: 0, attempts: [] } as unknown as PracticeSession
  const pending = [{ kind: 'reference', step: 0 }] as PracticeInput[]
  const state = sessionCard(session, pending)
  expect(state.revealed).toBe(true)
  expect(state.expanded).toBe(false)
  expect(state.originalResponse).toBeNull()
  expect(
    sessionCard(session, [
      ...pending,
      { kind: 'respond', step: 0, modality: 'recorded' } as PracticeInput,
    ]).originalResponse?.supported,
  ).toBe(true)
})

it('reconstructs the original recall before a pending answer and later take', () => {
  const session = { cursor: 0, attempts: [] } as unknown as PracticeSession
  const pending = [
    { kind: 'answer', step: 0, modality: 'self_report' },
    { kind: 'explain', step: 0 },
    { kind: 'respond', step: 0, modality: 'recorded' },
  ] as PracticeInput[]
  expect(sessionCard(session, pending).originalResponse).toEqual({
    modality: 'self_report',
    supported: false,
  })
})
