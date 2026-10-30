import { beforeEach, expect, it, vi } from 'vitest'
import { ApiError, api } from '../src/api'
import { usePractice } from '../src/composables/usePractice'
import { openedState } from '../src/practiceReducer'
import type { Card, PracticeState } from '../src/studyTypes'

vi.mock('../src/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../src/api')>()),
  api: vi.fn(),
}))

beforeEach(() => {
  vi.mocked(api).mockReset()
})

const progress = { due_completed: 1, due_allowance: 20, new_completed: 0, new_allowance: 5 }
const card = (token: string): Card => ({
  view_token: token,
  phrase_id: `phrase-${token}`,
  mode: 'daily',
  kind: 'recall',
  revealed: true,
  cue: { situation: 'Colleague asks: How is it going?', partner_line: null },
  other_cue_available: true,
  hint: null,
  answer: null,
  recordings: [],
})
const state = (current: Card | null): PracticeState => ({ card: current, progress, message: null })
const paths = () => vi.mocked(api).mock.calls.map(([path]) => path)
const posted = () => paths().filter((path) => path !== '/practice/current')
const operation = (call: number) =>
  JSON.parse(String(vi.mocked(api).mock.calls[call]![1]!.body)).operation_id

it('acknowledges a replayed completion without a current card and opens nothing', async () => {
  const practice = usePractice()
  practice.view.value = openedState(state(card('a')))
  vi.mocked(api)
    .mockImplementationOnce(async (_path, init) => ({
      receipt: { completion_id: JSON.parse(String(init!.body)).completion_id },
      replayed: true,
      state: null,
    }))
    .mockResolvedValueOnce(state(null))
  await practice.complete('got_it')
  expect(paths()).toEqual(['/practice/complete', '/practice/current'])
  expect(practice.view.value.card).toBeNull()
  expect(practice.view.value.message).toBe('Your answer was saved. Press Daily to continue.')
  expect(practice.error.value).toBe('')
})

it('acknowledges a replayed Extra when another tab owns the view', async () => {
  const practice = usePractice()
  vi.mocked(api)
    .mockRejectedValueOnce(new TypeError('Failed to fetch'))
    .mockImplementationOnce(async () => ({
      receipt: { operation_id: operation(0) },
      replayed: true,
      state: null,
    }))
    .mockResolvedValueOnce(state(null))
  await practice.openExtra(null)
  await practice.retryExtra()
  expect(operation(1)).toBe(operation(0))
  expect(paths()).toEqual(['/practice/extra', '/practice/extra', '/practice/current'])
  expect(practice.view.value.card).toBeNull()
  expect(practice.view.value.message).toBe('Extra was saved. Press Daily to continue.')
})

it('a conflicting hint shows the current card read without changing it', async () => {
  const practice = usePractice()
  practice.view.value = openedState(state(card('old')))
  vi.mocked(api)
    .mockRejectedValueOnce(new ApiError('This card is no longer current.', 409))
    .mockResolvedValueOnce(state(card('current')))
  await practice.hint()
  expect(posted()).toEqual(['/practice/hint'])
  expect(practice.view.value.card?.view_token).toBe('current')
  expect(practice.error.value).toContain('Showing the current card.')
})

it('a conflicting rating with no current regular card returns to entry', async () => {
  const practice = usePractice()
  practice.view.value = openedState(state(card('old')))
  vi.mocked(api)
    .mockRejectedValueOnce(new ApiError('This card is no longer current.', 409))
    .mockResolvedValueOnce(state(null))
  await practice.complete('effort')
  expect(posted()).toEqual(['/practice/complete'])
  expect(practice.view.value.card).toBeNull()
  expect(practice.error.value).toBe('This card changed in another tab or a new day began.')
})

it('a failed current-view read leaves an actionable error', async () => {
  const practice = usePractice()
  practice.view.value = openedState(state(card('old')))
  vi.mocked(api)
    .mockRejectedValueOnce(new ApiError('This card is no longer current.', 409))
    .mockRejectedValueOnce(new TypeError('Failed to fetch'))
  await practice.reveal()
  expect(posted()).toEqual(['/practice/reveal'])
  expect(practice.error.value).toBe('TypeError: Failed to fetch')
  expect(practice.busy.value).toBe(false)
})

it('any later open abandons a failed Extra press so its Retry cannot resend it', async () => {
  const practice = usePractice()
  vi.mocked(api)
    .mockRejectedValueOnce(new TypeError('Failed to fetch'))
    .mockResolvedValueOnce(state(card('daily')))
  await practice.openExtra(null)
  expect(practice.pendingExtra.value).not.toBeNull()
  await practice.openDaily(null)
  expect(practice.pendingExtra.value).toBeNull()
  vi.mocked(api)
    .mockRejectedValueOnce(new TypeError('Failed to fetch'))
    .mockRejectedValueOnce(new ApiError('Service unavailable', 503))
  await practice.openExtra(null)
  await practice.openDaily(null)
  expect(practice.pendingExtra.value).toBeNull()
  expect(practice.error.value).toBe('Error: Service unavailable')
  vi.mocked(api)
    .mockRejectedValueOnce(new TypeError('Failed to fetch'))
    .mockResolvedValueOnce(state(card('phrase')))
  await practice.openExtra(null)
  await practice.openPhrase('phrase-phrase')
  expect(practice.pendingExtra.value).toBeNull()
  await practice.retryExtra()
  expect(paths().filter((path) => path === '/practice/extra')).toHaveLength(3)
})

it('drops a completion result that arrives after the learner left its card', async () => {
  const practice = usePractice()
  practice.view.value = openedState(state(card('left')))
  let release!: (value: unknown) => void
  vi.mocked(api).mockReturnValueOnce(new Promise((resolve) => (release = resolve)))
  const completing = practice.complete('got_it')
  practice.closeCard()
  release({ receipt: { completion_id: 'other' }, replayed: false, state: state(card('next')) })
  await completing
  expect(paths()).toEqual(['/practice/complete'])
  expect(practice.view.value.card).toBeNull()
  expect(practice.error.value).toBe('')
  vi.mocked(api).mockRejectedValueOnce(new TypeError('Failed to fetch'))
  expect(await practice.openExtra(null)).toBe(false)
  expect(practice.view.value.card).toBeNull()
  expect(practice.pendingExtra.value).not.toBeNull()
})

it('reports a shown card only for the open that produced it', async () => {
  const practice = usePractice()
  vi.mocked(api)
    .mockResolvedValueOnce(state(card('daily')))
    .mockResolvedValueOnce(state(null))
  expect(await practice.openDaily(null)).toBe(true)
  expect(await practice.openPhrase('phrase-gone')).toBe(false)
  vi.mocked(api).mockRejectedValueOnce(new ApiError('Phrase not found', 404))
  practice.view.value = openedState(state(card('kept')))
  expect(await practice.openPhrase('phrase-gone')).toBe(false)
})
