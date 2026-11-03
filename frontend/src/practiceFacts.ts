import type { LibraryCounts, PracticeFacts, PracticeTotals, StudiedPhrase } from './factsTypes'

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

export function streakLine({ current }: PracticeFacts['streak']) {
  return current ? `🔥 ${current}-day practice streak` : 'No current practice streak'
}

export const bestLine = ({ best }: PracticeFacts['streak']) =>
  `Personal best: ${counted(best, 'day')}`

const titled = (label: string, parts: string[]) =>
  parts.length ? `${label} — ${parts.join(' / ')}` : label

export const todayTitle = (today: PracticeTotals) =>
  titled(
    'Today',
    present([
      [today.recalls, counted(today.recalls, 'recall')],
      [today.first_meetings, `${today.first_meetings} new`],
    ]),
  )

export const ratingsLine = (totals: PracticeTotals) =>
  present([
    [totals.got_it, `${totals.got_it} Got it`],
    [totals.effort, `${totals.effort} Effort`],
    [totals.not_yet, `${totals.not_yet} Not yet`],
    [totals.skipped, `${totals.skipped} skipped`],
  ]).join(' · ')

export const recordingsLine = (count: number) => `${counted(count, 'recording')} made`

export const progressTitle = ({ work }: PracticeFacts) =>
  titled('Progress', [counted(work.practice_days, 'day')])

export const progressLine = ({ work }: PracticeFacts) =>
  present([
    [work.recalls, counted(work.recalls, 'recall')],
    [work.first_meetings, counted(work.first_meetings, 'new phrase')],
  ]).join(' · ')

export const totalTitle = (library: LibraryCounts) =>
  titled('Total', [counted(library.phrase_count, 'phrase')])

export const libraryLine = (library: LibraryCounts) =>
  present([
    [library.due_count, `${library.due_count} due now`],
    [library.later_count, `${library.later_count} later`],
    [library.new_count, `${library.new_count} new`],
  ]).join(' · ')

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
