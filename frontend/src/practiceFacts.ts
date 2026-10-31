import type { PracticeFacts, StudiedPhrase, TodayCounts } from './factsTypes'

const weekdays = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
const ratings: Record<NonNullable<StudiedPhrase['rating']>, string> = {
  got_it: 'Got it',
  effort: 'Effort',
  not_yet: 'Not yet',
}
const modes: Record<StudiedPhrase['mode'], string> = {
  daily: 'Daily',
  extra: 'Extra',
  practice: 'Practice',
}

export const counted = (count: number, one: string, many = `${one}s`) =>
  `${count} ${count === 1 ? one : many}`

const present = (parts: [number, string][]) =>
  parts.filter(([count]) => count > 0).map(([, text]) => text)

// Learning dates are calendar days, so they are read in UTC to avoid the browser's offset.
const calendarDay = (date: string) => new Date(`${date}T00:00:00Z`)

export const weekday = (date: string) => weekdays[calendarDay(date).getUTCDay()]

export function nextDay(date: string) {
  const day = calendarDay(date)
  day.setUTCDate(day.getUTCDate() + 1)
  return day.toISOString().slice(0, 10)
}

export function streakLine({ current }: PracticeFacts['streak']) {
  return current ? `🔥 ${current}-day practice streak` : 'No current practice streak'
}

export const bestLine = ({ best }: PracticeFacts['streak']) =>
  `Personal best: ${counted(best, 'day')}`

export function todayLine(today: TodayCounts) {
  return [
    counted(today.completions, 'completion'),
    ...present([
      [today.first_meetings, counted(today.first_meetings, 'first meeting')],
      [today.skipped, `${today.skipped} skipped`],
    ]),
  ].join(' · ')
}

export const ratingsLine = (today: TodayCounts) =>
  present([
    [today.got_it, `${today.got_it} Got it`],
    [today.effort, `${today.effort} Effort`],
    [today.not_yet, `${today.not_yet} Not yet`],
  ]).join(' · ')

export const workLine = ({ work }: PracticeFacts) =>
  [
    counted(work.completions, 'completion'),
    counted(work.first_meetings, 'first meeting'),
    `${counted(work.practice_days, 'day')} with practice`,
  ].join(' · ')

export function nextReturnLine(facts: PracticeFacts) {
  const next = facts.next_return
  if (!next) return ''
  const when =
    next.learning_date === facts.learning_date
      ? 'Today'
      : next.learning_date === nextDay(facts.learning_date)
        ? 'Tomorrow'
        : next.learning_date
  return `Next return: ${when} · ${counted(next.phrase_count, 'phrase')} due`
}

export function studiedLine(item: StudiedPhrase) {
  const outcome =
    item.kind === 'first_study'
      ? 'first meeting'
      : item.skipped
        ? 'Skip'
        : ratings[item.rating ?? 'not_yet']
  const parts = [outcome, modes[item.mode], item.lesson_title]
  if (!item.phrase_id) parts.push('deleted')
  return parts.join(' · ')
}
