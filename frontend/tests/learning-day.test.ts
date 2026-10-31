import { nextTick, ref } from 'vue'
import { beforeEach, expect, test, vi } from 'vitest'

beforeEach(() => vi.resetModules())
const fresh = () => import('../src/learningDay')

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

test('a message stays hidden on later learning dates until a new one appears', async () => {
  const { observeLearningDay, useDatedMessage } = await fresh()
  observeLearningDay('2026-11-02')
  const message = ref('')
  const shown = useDatedMessage(() => message.value)
  message.value = "Today's Daily practice is complete."
  await nextTick()
  expect(shown.value).toBe("Today's Daily practice is complete.")
  observeLearningDay('2026-11-03')
  expect(shown.value).toBe('')
  message.value = 'Nothing waiting today.'
  await nextTick()
  expect(shown.value).toBe('Nothing waiting today.')
})

test('a message dated before a remount stays hidden when the entry is shown again', async () => {
  const { observeLearningDay, useDatedMessage } = await fresh()
  observeLearningDay('2026-11-02')
  const message = ref("Today's Daily practice is complete.")
  const shown = useDatedMessage(() => message.value)
  observeLearningDay('2026-11-03')
  observeLearningDay('2026-11-03')
  expect(shown.value).toBe('')
})
