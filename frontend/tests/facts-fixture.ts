import type { PracticeFacts, StudiedPhrase } from '../src/factsTypes'

export const today = {
  completions: 27,
  new_attempts: 5,
  first_meetings: 0,
  recalls: 20,
  skipped: 2,
  easy: 0,
  got_it: 18,
  effort: 2,
  not_yet: 0,
  recordings: 30,
}
export const library = {
  phrase_count: 131,
  due_count: 72,
  later_count: 39,
  new_count: 20,
  long_term_count: 12,
}
export const facts = (changes: Partial<PracticeFacts> = {}): PracticeFacts => ({
  learning_date: '2026-10-31',
  streak: { current: 8, best: 12, recent_days: [] },
  week: { practice_days: 2, target: 3 },
  today,
  work: { ...today, recalls: 45, new_attempts: 112, got_it: 39, effort: 6, practice_days: 3 },
  library,
  notices: [],
  ...changes,
})
export const item = (changes: Partial<StudiedPhrase>): StudiedPhrase => ({
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
