import { expect, it } from 'vitest'
import type { PracticeFacts, StudiedPhrase } from '../src/factsTypes'
import {
  bestLine,
  ratingsLine,
  streakLine,
  studiedLine,
  todayLine,
  waitingLine,
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
  waiting: { new_count: 10, due_count: 72 },
  notices: [],
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

it('states how many phrases are new and due right now', () => {
  expect(waitingLine(facts())).toBe('10 phrases new · 72 phrases due')
  expect(waitingLine(facts({ waiting: { new_count: 1, due_count: 0 } }))).toBe(
    '1 phrase new · 0 phrases due',
  )
})

it('names calendar weekdays regardless of the browser offset', () => {
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
