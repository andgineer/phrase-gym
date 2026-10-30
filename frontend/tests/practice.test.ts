import { beforeEach, expect, it, vi } from 'vitest'
import { ApiError, api } from '../src/api'
import { usePractice } from '../src/composables/usePractice'
import { completedPractice, openedState, progressLine, updatedCard } from '../src/practiceReducer'
import type { Card, CompletionResult, PracticeState } from '../src/studyTypes'

vi.mock('../src/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../src/api')>()),
  api: vi.fn(),
}))

beforeEach(() => {
  vi.mocked(api).mockReset()
})

const progress = { due_completed: 3, due_allowance: 20, new_completed: 0, new_allowance: 5 }
function card(token: string, changes: Partial<Card> = {}): Card {
  return {
    view_token: token,
    phrase_id: `phrase-${token}`,
    mode: 'daily',
    kind: 'recall',
    revealed: false,
    cue: { situation: 'Colleague asks: How is it going?', partner_line: null },
    other_cue_available: true,
    hint: null,
    answer: null,
    recordings: [],
    ...changes,
  }
}
const state = (current: Card | null, message: string | null = null): PracticeState => ({
  card: current,
  progress,
  message,
})
const body = (call: number) => JSON.parse(String(vi.mocked(api).mock.calls[call]![1]!.body))
const paths = () => vi.mocked(api).mock.calls.map(([path]) => path)

it('ignores a late card for a superseded appearance and reconciles replayed completions', () => {
  const view = openedState(state(card('current')))
  expect(updatedCard(view, card('old', { hint: 'far' }))).toBe(view)
  expect(updatedCard(view, card('current', { hint: 'far' }), true)).toMatchObject({
    detailsOpen: true,
    card: { hint: 'far' },
  })
  const pending = {
    completion_id: 'done-1',
    view_token: 'current',
    phrase_id: 'phrase-current',
    outcome: 'got_it' as const,
  }
  const receipt = { completion_id: 'done-1' } as CompletionResult['receipt']
  const next = state(card('next'))
  expect(completedPractice(view, pending, { receipt, replayed: true, state: null })).toEqual({
    view,
    reconcile: true,
  })
  const moved = openedState(state(card('newer')))
  expect(
    completedPractice(moved, pending, { receipt, replayed: false, state: next }).reconcile,
  ).toBe(true)
  expect(completedPractice(view, pending, { receipt, replayed: false, state: next })).toEqual({
    view: openedState(next),
    reconcile: false,
  })
})

it('labels progress by mode and card kind', () => {
  expect(progressLine(card('a'), progress)).toBe('Daily · 4 / 20 due')
  expect(progressLine(card('a', { kind: 'first_study' }), progress)).toBe('Daily · 1 / 5 new')
})

it('completes once while saving and advances to the returned card', async () => {
  const practice = usePractice()
  practice.view.value = openedState(state(card('first', { revealed: true })))
  let release!: (value: CompletionResult) => void
  vi.mocked(api).mockReturnValueOnce(new Promise((resolve) => (release = resolve)))
  const first = practice.complete('got_it')
  await practice.complete('not_yet')
  expect(api).toHaveBeenCalledTimes(1)
  const request = body(0)
  expect(request).toMatchObject({
    view_token: 'first',
    phrase_id: 'phrase-first',
    outcome: 'got_it',
  })
  release({
    receipt: { completion_id: request.completion_id } as CompletionResult['receipt'],
    replayed: false,
    state: state(card('second')),
  })
  await first
  expect(practice.view.value.card?.view_token).toBe('second')
  expect(practice.busy.value).toBe(false)
})

it('keeps the card after a failed save and retries with the same completion ID', async () => {
  const practice = usePractice()
  practice.view.value = openedState(state(card('first', { revealed: true })))
  vi.mocked(api).mockRejectedValueOnce(new ApiError('Service unavailable', 503))
  await practice.complete('effort')
  expect(practice.view.value.card?.view_token).toBe('first')
  expect(practice.error.value).toContain('retry to save it')
  const id = body(0).completion_id
  vi.mocked(api).mockResolvedValueOnce({
    receipt: { completion_id: id },
    replayed: false,
    state: state(null, "Today's Daily practice is complete."),
  })
  await practice.retry()
  expect(body(1)).toEqual(body(0))
  expect(practice.view.value.card).toBeNull()
  expect(practice.view.value.message).toBe("Today's Daily practice is complete.")
  expect(practice.pending.value).toBeNull()
})

it('reconciles a stale view conflict by reading the current view, never reopening Daily', async () => {
  const practice = usePractice()
  vi.mocked(api).mockResolvedValueOnce(state(card('first', { revealed: true })))
  await practice.openDaily('lesson-1')
  vi.mocked(api)
    .mockRejectedValueOnce(new ApiError('This card is no longer current.', 409))
    .mockResolvedValueOnce(state(card('replacement')))
  await practice.complete('skipped')
  expect(paths()).toEqual(['/practice/daily', '/practice/complete', '/practice/current'])
  expect(vi.mocked(api).mock.calls[2]![1]).toBeUndefined()
  expect(practice.view.value.card?.view_token).toBe('replacement')
  expect(practice.error.value).toContain('changed in another tab')
  expect(practice.pending.value).toBeNull()
})

it('sends new operation IDs for cue and hint and applies only the open card', async () => {
  const practice = usePractice()
  practice.view.value = openedState(state(card('first')))
  vi.mocked(api)
    .mockResolvedValueOnce(card('first', { hint: 'far' }))
    .mockResolvedValueOnce(
      card('first', { hint: 'far', cue: { situation: 'Friend asks.', partner_line: null } }),
    )
  await practice.hint()
  await practice.newCue()
  expect(paths()).toEqual(['/practice/hint', '/practice/cue'])
  expect(body(0).operation_id).not.toBe(body(1).operation_id)
  expect(practice.view.value.card).toMatchObject({
    hint: 'far',
    cue: { situation: 'Friend asks.' },
  })
})
