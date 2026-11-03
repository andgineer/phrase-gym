import { expect, it } from 'vitest'
import type { PracticeFacts, StudiedPhrase } from '../src/factsTypes'
import {
  bestLine,
  libraryLine,
  progressLine,
  progressTitle,
  ratingsLine,
  recordingsLine,
  streakLine,
  studiedLine,
  todayTitle,
  totalTitle,
} from '../src/practiceFacts'

const today = {
  completions: 27,
  first_meetings: 5,
  recalls: 20,
  skipped: 2,
  got_it: 18,
  effort: 2,
  not_yet: 0,
  recordings: 30,
}
const library = {
  phrase_count: 131,
  due_count: 72,
  later_count: 39,
  new_count: 20,
  long_term_count: 12,
}
const facts = (changes: Partial<PracticeFacts> = {}): PracticeFacts => ({
  learning_date: '2026-10-31',
  streak: { current: 8, best: 12, recent_days: [] },
  today,
  work: { ...today, recalls: 45, first_meetings: 112, got_it: 39, effort: 6, practice_days: 3 },
  library,
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

it('titles Today with its recalls and new phrases and lists ratings without empty parts', () => {
  expect(todayTitle(today)).toBe('Today — 20 recalls / 5 new')
  expect(todayTitle({ ...today, recalls: 1, first_meetings: 0 })).toBe('Today — 1 recall')
  expect(todayTitle({ ...today, recalls: 0, first_meetings: 0 })).toBe('Today')
  expect(ratingsLine(today)).toBe('18 Got it · 2 Effort · 2 skipped')
  expect(ratingsLine({ ...today, got_it: 0, effort: 0, skipped: 0 })).toBe('')
  expect(recordingsLine(30)).toBe('30 recordings made')
  expect(recordingsLine(1)).toBe('1 recording made')
})

it('titles Progress with practice days and lists what was done', () => {
  expect(progressTitle(facts())).toBe('Progress — 3 days')
  expect(progressLine(facts())).toBe('45 recalls · 112 new phrases')
  const first = facts({
    work: { ...facts().work, recalls: 0, first_meetings: 1, practice_days: 1 },
  })
  expect(progressTitle(first)).toBe('Progress — 1 day')
  expect(progressLine(first)).toBe('1 new phrase')
})

it('titles Total with the library size and splits it into due now, later and new', () => {
  expect(totalTitle(library)).toBe('Total — 131 phrases')
  expect(libraryLine(library)).toBe('72 due now · 39 later · 20 new')
  expect(libraryLine({ ...library, due_count: 0, new_count: 0 })).toBe('39 later')
  expect(totalTitle({ ...library, phrase_count: 1 })).toBe('Total — 1 phrase')
})

it('states the streak factually, including after a missed day', () => {
  expect(streakLine(facts().streak)).toBe('🔥 8-day practice streak')
  expect(bestLine(facts().streak)).toBe('Personal best: 12 days')
  const missed = { current: 0, best: 1, recent_days: [] }
  expect(streakLine(missed)).toBe('No current practice streak')
  expect(bestLine(missed)).toBe('Personal best: 1 day')
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
