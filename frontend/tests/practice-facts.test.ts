import { expect, it } from 'vitest'
import { today, library, facts, item } from './facts-fixture'
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
  weekLine,
} from '../src/practiceFacts'

it('titles Today with its recalls and new phrases and lists ratings without empty parts', () => {
  expect(todayTitle(today)).toBe('Today — 20 recalls / 5 new')
  expect(todayTitle({ ...today, recalls: 1, new_attempts: 0 })).toBe('Today — 1 recall')
  expect(todayTitle({ ...today, recalls: 0, new_attempts: 0 })).toBe('Today')
  expect(ratingsLine(today)).toBe('18 Good · 2 Hard · 2 skipped')
  expect(ratingsLine({ ...today, got_it: 0, effort: 0, skipped: 0 })).toBe('')
  expect(recordingsLine(30)).toBe('30 recordings made')
  expect(recordingsLine(1)).toBe('1 recording made')
})

it('titles Progress with practice days and lists what was done', () => {
  expect(progressTitle(facts())).toBe('Progress — 3 days')
  expect(progressLine(facts())).toBe('45 recalls · 112 new phrases')
  const first = facts({
    work: { ...facts().work, recalls: 0, new_attempts: 1, practice_days: 1 },
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

it('states this week’s practice days against a chosen, unpaused target without debt', () => {
  expect(weekLine(facts().week)).toBe('This week: 2 of 3 days')
  expect(weekLine({ practice_days: 0, target: 1 })).toBe('This week: 0 of 1 day')
  expect(weekLine({ practice_days: 3, target: 3 })).toBe('This week: 3 days · target reached')
  expect(weekLine({ practice_days: 1, target: 1 })).toBe('This week: 1 day · target reached')
  expect(weekLine({ practice_days: 5, target: 3 })).toBe('This week: 5 days · target reached')
  // No target chosen, or the chosen one paused: nothing is shown.
  expect(weekLine({ practice_days: 4, target: null })).toBe('')
})

it('labels each studied phrase with its outcome, mode, lesson and deletion', () => {
  expect(studiedLine(item({ kind: 'first_study', rating: null }))).toBe(
    'historical first meeting · Daily · Small talk',
  )
  expect(studiedLine(item({ mode: 'extra' }))).toBe('Hard · Extra · Small talk')
  expect(studiedLine(item({ skipped: true, rating: null, phrase_id: null }))).toBe(
    'Skip · Daily · Small talk · deleted',
  )
  expect(studiedLine(item({ mode: 'practice', rating: 'got_it' }))).toBe(
    'Good · Practice · Small talk',
  )
})

it('includes Easy in totals and studied history', () => {
  expect(ratingsLine({ ...today, easy: 3 })).toBe('3 Easy · 18 Good · 2 Hard · 2 skipped')
  expect(studiedLine(item({ rating: 'easy' }))).toBe('Easy · Daily · Small talk')
})

it('keeps historical ungraded first meetings separate from rated new attempts', () => {
  const history = { ...today, recalls: 0, new_attempts: 1, first_meetings: 2 }
  expect(todayTitle(history)).toBe('Today — 1 new / 2 historical first meetings')
  expect(progressLine(facts({ work: { ...history, practice_days: 1 } }))).toBe(
    '1 new phrase · 2 historical first meetings',
  )
  expect(studiedLine(item({ kind: 'new', rating: 'not_yet' }))).toBe('Again · Daily · Small talk')
})
