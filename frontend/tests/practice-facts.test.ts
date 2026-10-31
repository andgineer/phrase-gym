import { expect, it } from 'vitest'
import type { PracticeFacts, StudiedPhrase } from '../src/factsTypes'
import {
  bestLine,
  nextDay,
  nextReturnLine,
  ratingsLine,
  streakLine,
  studiedLine,
  todayLine,
  weekday,
  workLine,
} from '../src/practiceFacts'

const today = {
  completions: 18,
  first_meetings: 6,
  recalls: 12,
  skipped: 2,
  got_it: 6,
  effort: 3,
  not_yet: 1,
  recordings: 5,
}
const facts = (changes: Partial<PracticeFacts> = {}): PracticeFacts => ({
  learning_date: '2026-10-31',
  streak: { current: 8, best: 12, recent_days: [] },
  today,
  work: { completions: 340, first_meetings: 126, practice_days: 17 },
  next_return: { learning_date: '2026-11-01', phrase_count: 4 },
  ...changes,
})
const item = (changes: Partial<StudiedPhrase>): StudiedPhrase => ({
  completion_id: 'completion-1',
  phrase_id: 'phrase-1',
  text: 'So far, so good.',
  lesson_title: 'Small talk',
  mode: 'daily',
  kind: 'recall',
  rating: 'effort',
  skipped: false,
  ...changes,
})

it('describes Today counts, ratings and cumulative work without empty parts', () => {
  expect(todayLine(today)).toBe('18 completions · 6 first meetings · 2 skipped')
  expect(ratingsLine(today)).toBe('6 Got it · 3 Effort · 1 Not yet')
  const single = { ...today, completions: 1, first_meetings: 1, skipped: 0, got_it: 0 }
  expect(todayLine(single)).toBe('1 completion · 1 first meeting')
  expect(ratingsLine({ ...single, effort: 0, not_yet: 0 })).toBe('')
  expect(workLine(facts())).toBe('340 completions · 126 first meetings · 17 days with practice')
})

it('states the streak factually, including after a missed day', () => {
  expect(streakLine(facts().streak)).toBe('🔥 8-day practice streak')
  expect(bestLine(facts().streak)).toBe('Personal best: 12 days')
  const missed = { current: 0, best: 1, recent_days: [] }
  expect(streakLine(missed)).toBe('No current practice streak')
  expect(bestLine(missed)).toBe('Personal best: 1 day')
})

it('names tomorrow and calendar weekdays regardless of the browser offset', () => {
  expect(nextReturnLine(facts())).toBe('Next return: Tomorrow · 4 phrases due')
  const later = { learning_date: '2026-11-05', phrase_count: 1 }
  expect(nextReturnLine(facts({ next_return: later }))).toBe(
    'Next return: 2026-11-05 · 1 phrase due',
  )
  expect(nextReturnLine(facts({ next_return: null }))).toBe('')
  const waiting = { learning_date: '2026-10-31', phrase_count: 2 }
  expect(nextReturnLine(facts({ next_return: waiting }))).toBe('Next return: Today · 2 phrases due')
  expect(nextDay('2026-12-31')).toBe('2027-01-01')
  expect(nextDay('2026-03-29')).toBe('2026-03-30')
  expect(weekday('2026-10-31')).toBe('Sat')
})

it('labels each studied phrase with its outcome, mode, lesson and deletion', () => {
  expect(studiedLine(item({ kind: 'first_study', rating: null }))).toBe(
    'first meeting · Daily · Small talk',
  )
  expect(studiedLine(item({ mode: 'extra' }))).toBe('Effort · Extra · Small talk')
  expect(studiedLine(item({ skipped: true, rating: null, phrase_id: null }))).toBe(
    'Skip · Daily · Small talk · deleted',
  )
  expect(studiedLine(item({ mode: 'practice', rating: 'got_it' }))).toBe(
    'Got it · Practice · Small talk',
  )
})
