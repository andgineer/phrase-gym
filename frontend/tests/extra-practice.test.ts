import { beforeEach, expect, it, vi } from 'vitest'
import { ApiError, api } from '../src/api'
import { usePractice } from '../src/composables/usePractice'
import { extendedPractice, initialView, openedState, progressLine } from '../src/practiceReducer'
import type { Card, ExtraResult, PracticeState } from '../src/studyTypes'

vi.mock('../src/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../src/api')>()),
  api: vi.fn(),
}))

// A returned mock would be run by Vitest as an after-each cleanup hook.
beforeEach(() => {
  vi.mocked(api).mockReset()
})

const progress = { due_completed: 20, due_allowance: 40, new_completed: 5, new_allowance: 10 }
function card(token: string, changes: Partial<Card> = {}): Card {
  return {
    view_token: token,
    phrase_id: `phrase-${token}`,
    mode: 'extra',
    kind: 'recall',
    revealed: true,
    cue: { situation: 'Colleague asks: How is it going?', partner_line: null },
    other_cue_available: false,
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
const receipt = (operation_id: string) => ({
  operation_id,
  learning_date: '2026-10-28',
  due_target: 40,
  new_target: 10,
})
const extraResult = (operation_id: string, next: Card | null): ExtraResult => ({
  receipt: receipt(operation_id),
  replayed: false,
  state: state(next),
})
const body = (call: number) => JSON.parse(String(vi.mocked(api).mock.calls[call]![1]!.body))
const paths = () => vi.mocked(api).mock.calls.map(([path]) => path)

it('labels Extra progress and shows Practice alone', () => {
  expect(progressLine(card('a'), progress)).toBe('Extra · 21 / 40 due')
  expect(progressLine(card('a', { kind: 'first_study' }), progress)).toBe('Extra · 6 / 10 new')
  expect(progressLine(card('a', { mode: 'practice' }), progress)).toBe('Practice')
})

it('applies a fresh Extra result and reconciles a replayed one', () => {
  const view = openedState(state(card('current')))
  expect(extendedPractice(view, extraResult('op-1', card('next')))).toEqual({
    view: openedState(state(card('next'))),
    reconcile: false,
  })
  const replay = { receipt: receipt('op-1'), replayed: true, state: null }
  expect(extendedPractice(view, replay)).toEqual({ view, reconcile: true })
})

it('gives every deliberate Extra press its own operation ID', async () => {
  const practice = usePractice()
  vi.mocked(api).mockImplementation(async (_path, init) =>
    extraResult(JSON.parse(String(init!.body)).operation_id, card('x')),
  )
  await practice.openExtra('lesson-1')
  await practice.openExtra('lesson-1')
  expect(paths()).toEqual(['/practice/extra', '/practice/extra'])
  expect(body(0).lesson_id).toBe('lesson-1')
  expect(body(0).operation_id).not.toBe(body(1).operation_id)
  expect(practice.pendingExtra.value).toBeNull()
})

it('retries a lost Extra response with the same ID and reconciles the replay', async () => {
  const practice = usePractice()
  vi.mocked(api).mockRejectedValueOnce(new TypeError('Failed to fetch'))
  await practice.openExtra(null)
  expect(practice.error.value).toContain('retry it')
  expect(practice.pendingExtra.value).not.toBeNull()
  const id = body(0).operation_id
  vi.mocked(api)
    .mockResolvedValueOnce({ receipt: receipt(id), replayed: true, state: null })
    .mockResolvedValueOnce(state(card('newer')))
  await practice.retryExtra()
  expect(paths()).toEqual(['/practice/extra', '/practice/extra', '/practice/current'])
  expect(body(1)).toEqual(body(0))
  expect(practice.view.value.card?.view_token).toBe('newer')
  expect(practice.pendingExtra.value).toBeNull()
  expect(practice.error.value).toBe('')
})

it('drops a rejected Extra press and explains an empty result at once', async () => {
  const practice = usePractice()
  vi.mocked(api).mockRejectedValueOnce(new ApiError('Lesson not found', 404))
  await practice.openExtra('gone')
  expect(practice.pendingExtra.value).toBeNull()
  expect(practice.error.value).toBe('Error: Lesson not found')
  vi.mocked(api).mockImplementationOnce(async (_path, init) => ({
    ...extraResult(JSON.parse(String(init!.body)).operation_id, null),
    state: state(null, 'Nothing waiting today. Next due: 2026-10-29.'),
  }))
  await practice.openExtra(null)
  expect(practice.view.value.card).toBeNull()
  expect(practice.view.value.message).toBe('Nothing waiting today. Next due: 2026-10-29.')
})

it('opens phrase Practice and closes its card after the rating', async () => {
  const practice = usePractice()
  vi.mocked(api).mockResolvedValueOnce(state(card('p', { mode: 'practice' })))
  await practice.openPhrase('phrase-p')
  expect(body(0)).toEqual({ phrase_id: 'phrase-p' })
  expect(practice.view.value.card?.mode).toBe('practice')
  vi.mocked(api).mockImplementationOnce(async (_path, init) => ({
    receipt: { completion_id: JSON.parse(String(init!.body)).completion_id },
    replayed: false,
    state: state(null),
  }))
  await practice.complete('got_it')
  expect(practice.view.value).toEqual(openedState(state(null)))
})

it('closes a stale or replayed Practice card without opening Daily', async () => {
  const practice = usePractice()
  practice.view.value = openedState(state(card('p', { mode: 'practice' })))
  vi.mocked(api).mockRejectedValueOnce(new ApiError('This card is no longer current.', 409))
  await practice.complete('effort')
  expect(paths()).toEqual(['/practice/complete'])
  expect(practice.view.value).toEqual(initialView())
  expect(practice.error.value).toContain('Practice card changed')
  practice.view.value = openedState(state(card('q', { mode: 'practice' })))
  vi.mocked(api).mockImplementationOnce(async (_path, init) => ({
    receipt: { completion_id: JSON.parse(String(init!.body)).completion_id },
    replayed: true,
    state: null,
  }))
  await practice.complete('skipped')
  expect(paths()).toEqual(['/practice/complete', '/practice/complete'])
  expect(practice.view.value.card).toBeNull()
  expect(practice.error.value).toBe('')
})
