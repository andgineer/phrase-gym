import { expect, it } from 'vitest'
import { sessionCard } from '../src/practiceState'
import { language } from '../src/api'
import type { PracticeSession } from '../src/practiceTypes'
it('shows only acknowledged evidence for the addressed card', () => {
  const session = {
    cursor: 1,
    attempts: [{ step: 0, revealed: true }],
  } as unknown as PracticeSession
  expect(sessionCard(session)).toEqual({
    revealed: false,
    expanded: false,
    exposedBeforeResponse: false,
    originalResponse: null,
  })
})
it('later help does not relabel the server original response', () => {
  const session = {
    cursor: 0,
    attempts: [
      {
        step: 0,
        revealed: true,
        expanded: true,
        supported: true,
        original: { modality: 'recorded', supported: false },
      },
    ],
  } as unknown as PracticeSession
  const card = sessionCard(session)
  expect(card.revealed).toBe(true)
  expect(card.expanded).toBe(true)
  expect(card.originalResponse).toEqual({ modality: 'recorded', supported: false })
})
it('rejects invalid learning languages without substitution', () => {
  expect(language('sr')).toBe('sr')
  expect(language('unknown')).toBeNull()
  expect(language(['en', 'sr'])).toBeNull()
})
