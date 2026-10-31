import { nextTick, ref } from 'vue'
import { beforeEach, expect, test, vi } from 'vitest'

beforeEach(() => vi.resetModules())
const fresh = () => import('../src/learningDay')
const complete = "Today's Daily practice is complete."

test('a date change is signalled once, never for the first known or same date', async () => {
  const { observeLearningDay, onLearningDayChange } = await fresh()
  const refresh = vi.fn()
  onLearningDayChange(refresh)
  expect(observeLearningDay('2026-11-02')).toBe(false)
  expect(observeLearningDay('2026-11-02')).toBe(false)
  await nextTick()
  expect(refresh).not.toHaveBeenCalled()
  expect(observeLearningDay('2026-11-03')).toBe(true)
  await nextTick()
  expect(refresh).toHaveBeenCalledTimes(1)
})

test('an undated message shows until the next server date dates it', async () => {
  const { observeLearningDay, useDatedMessage } = await fresh()
  const state = ref({ message: '' })
  const shown = useDatedMessage(() => state.value)
  state.value = { message: complete }
  await nextTick()
  expect(shown.value).toBe(complete)
  observeLearningDay('2026-11-02')
  await nextTick()
  expect(shown.value).toBe(complete)
})

test('a message made after a stale known day takes the date reported next', async () => {
  const { observeLearningDay, useDatedMessage } = await fresh()
  observeLearningDay('2026-11-02')
  const state = ref({ message: '' })
  const shown = useDatedMessage(() => state.value)
  state.value = { message: complete }
  await nextTick()
  observeLearningDay('2026-11-03')
  await nextTick()
  expect(shown.value).toBe(complete)
  observeLearningDay('2026-11-03')
  await nextTick()
  expect(shown.value).toBe(complete)
})

test('a message dated an earlier day stays hidden until a new practice state', async () => {
  const { observeLearningDay, useDatedMessage } = await fresh()
  const state = ref({ message: '' })
  const shown = useDatedMessage(() => state.value)
  state.value = { message: 'Nothing waiting today.' }
  await nextTick()
  observeLearningDay('2026-11-02')
  await nextTick()
  observeLearningDay('2026-11-03')
  await nextTick()
  expect(shown.value).toBe('')
  state.value = { message: 'Nothing waiting today.' }
  await nextTick()
  expect(shown.value).toBe('Nothing waiting today.')
  observeLearningDay('2026-11-03')
  await nextTick()
  expect(shown.value).toBe('Nothing waiting today.')
})
