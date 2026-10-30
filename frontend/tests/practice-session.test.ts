import { nextTick, ref } from 'vue'
import { beforeEach, expect, it, vi } from 'vitest'
import { ApiError, api } from '../src/api'
import { usePracticeSession } from '../src/composables/usePracticeSession'
import { openedState } from '../src/practiceReducer'
import type { Card, PracticeState } from '../src/studyTypes'

const media = vi.hoisted(() => ({
  capture: null as unknown as {
    capturing: { value: boolean }
    processing: { value: boolean }
    finish: () => void
  },
  stopPlayback: vi.fn(),
}))

vi.mock('../src/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../src/api')>()),
  api: vi.fn(),
}))
vi.mock('../src/takeTransfers', () => ({
  takeTransfers: { retain: vi.fn(), flush: vi.fn(async () => {}) },
}))
vi.mock('../src/composables/usePlayback', () => ({
  usePlayback: () => ({
    play: vi.fn(),
    compare: vi.fn(),
    compareTake: vi.fn(),
    stop: media.stopPlayback,
    active: ref(false),
    error: ref(''),
  }),
}))
// The recorder's stop event clears capturing before the final take is retained.
vi.mock('../src/composables/useCapture', () => ({
  useCapture: () => {
    const capturing = ref(false)
    const processing = ref(false)
    let stopping: Promise<void> | null = null
    let finish = () => {}
    media.capture = { capturing, processing, finish: () => finish() }
    return {
      capturing,
      processing,
      requesting: ref(false),
      error: ref(''),
      start: vi.fn(),
      release: vi.fn(),
      stop: async () => {
        if (capturing.value) {
          capturing.value = false
          processing.value = true
          stopping = new Promise<void>((resolve) => {
            finish = () => {
              processing.value = false
              stopping = null
              resolve()
            }
          })
        }
        if (stopping) await stopping
      },
    }
  },
}))

beforeEach(() => {
  vi.mocked(api).mockReset()
  media.stopPlayback.mockReset()
})

const progress = { due_completed: 0, due_allowance: 20, new_completed: 0, new_allowance: 5 }
const card = (token: string, revealed = true): Card => ({
  view_token: token,
  phrase_id: `phrase-${token}`,
  mode: 'daily',
  kind: 'recall',
  revealed,
  cue: { situation: 'Colleague asks: How is it going?', partner_line: null },
  other_cue_available: false,
  hint: null,
  answer: null,
  recordings: [],
})
const state = (current: Card | null): PracticeState => ({ card: current, progress, message: null })
const completions = () =>
  vi
    .mocked(api)
    .mock.calls.filter(([path]) => path === '/practice/complete')
    .map(([, init]) => JSON.parse(String(init!.body)))

function answerCompletion(next: Card | null) {
  vi.mocked(api).mockImplementation(async (path, init) => {
    if (path !== '/practice/complete') throw new Error(`unexpected ${path}`)
    const request = JSON.parse(String(init!.body))
    return {
      receipt: { completion_id: request.completion_id },
      replayed: false,
      state: state(next),
    }
  })
}

it('a second tap while the take is finishing sends one completion for the original card', async () => {
  const session = usePracticeSession()
  session.practice.view.value = openedState(state(card('a')))
  media.capture.capturing.value = true
  answerCompletion(card('b', false))
  const first = session.complete('skipped')
  const second = session.complete('skipped')
  await second
  expect(session.completing.value).toBe(true)
  expect(completions()).toEqual([])
  media.capture.finish()
  await first
  expect(completions().map((request) => [request.view_token, request.outcome])).toEqual([
    ['a', 'skipped'],
  ])
  expect(session.practice.view.value.card?.view_token).toBe('b')
  expect(session.completing.value).toBe(false)
})

it('waits for a stop already in progress before sending the rating', async () => {
  const session = usePracticeSession()
  session.practice.view.value = openedState(state(card('a')))
  media.capture.capturing.value = true
  const stopped = session.capture.stop()
  expect(media.capture.processing.value).toBe(true)
  answerCompletion(null)
  const rating = session.complete('got_it')
  await Promise.resolve()
  expect(completions()).toEqual([])
  media.capture.finish()
  await Promise.all([stopped, rating])
  expect(completions().map((request) => [request.view_token, request.outcome])).toEqual([
    ['a', 'got_it'],
  ])
})

it('drops a completion whose card was replaced while the take was finishing', async () => {
  const session = usePracticeSession()
  session.practice.view.value = openedState(state(card('a')))
  media.capture.capturing.value = true
  answerCompletion(null)
  const rating = session.complete('effort')
  session.practice.view.value = openedState(state(card('replacement', false)))
  media.capture.finish()
  await rating
  expect(completions()).toEqual([])
  expect(session.completing.value).toBe(false)
})

it('cancels old-phrase playback when a conflict reconciles to another card', async () => {
  const session = usePracticeSession()
  session.practice.view.value = openedState(state(card('a', false)))
  await nextTick()
  media.stopPlayback.mockClear()
  vi.mocked(api)
    .mockRejectedValueOnce(new ApiError('This card is no longer current.', 409))
    .mockResolvedValueOnce(state(card('b', false)))
  await session.practice.hint()
  await nextTick()
  expect(session.practice.view.value.card?.view_token).toBe('b')
  expect(media.stopPlayback).toHaveBeenCalled()
})
