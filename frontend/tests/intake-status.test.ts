import { ref, nextTick } from 'vue'
import { afterEach, expect, test, vi } from 'vitest'
import { useIntakeStatusPolling } from '../src/composables/useIntakeStatusPolling'
import { connectionRetrying, serverUnavailable } from '../src/api'
import type { components } from '../src/api.generated'

const cleanup = vi.hoisted(() => [] as (() => void)[])
vi.mock('vue', async () => ({
  ...(await vi.importActual('vue')),
  onUnmounted: (callback: () => void) => cleanup.push(callback),
}))
afterEach(() => {
  cleanup.splice(0).forEach((stop) => stop())
  vi.useRealTimers()
  serverUnavailable.value = connectionRetrying.value = false
})

test('only acknowledged pending work polls, backs off and stops after publication or leaving', async () => {
  vi.useFakeTimers()
  const id = ref<string>()
  const status = ref<components['schemas']['LessonStatus']>()
  const refresh = vi.fn(async () => {})
  useIntakeStatusPolling(() => id.value, status, refresh)
  await vi.advanceTimersByTimeAsync(10000)
  expect(refresh).not.toHaveBeenCalled()
  id.value = 'accepted'
  await nextTick()
  await vi.advanceTimersByTimeAsync(2000)
  expect(refresh).toHaveBeenCalledTimes(1)
  serverUnavailable.value = true
  await vi.advanceTimersByTimeAsync(4000)
  expect(refresh).toHaveBeenCalledTimes(1)
  serverUnavailable.value = false
  refresh.mockImplementation(async () => {
    status.value = { status: 'published' } as components['schemas']['LessonStatus']
  })
  await vi.advanceTimersByTimeAsync(8000)
  expect(refresh).toHaveBeenCalledTimes(2)
  await vi.advanceTimersByTimeAsync(30000)
  expect(refresh).toHaveBeenCalledTimes(2)
  id.value = 'other'
  await nextTick()
  cleanup.forEach((stop) => stop())
  await vi.advanceTimersByTimeAsync(20000)
  expect(refresh).toHaveBeenCalledTimes(2)
})

test('retrying failed preparation resumes automatic status checks for the same generation', async () => {
  vi.useFakeTimers()
  const id = ref<string>()
  const status = ref({ status: 'failed' } as components['schemas']['LessonStatus'])
  const refresh = vi.fn(async () => {})
  useIntakeStatusPolling(() => id.value, status, refresh)
  id.value = 'accepted'
  await nextTick()
  await vi.advanceTimersByTimeAsync(10000)
  expect(refresh).not.toHaveBeenCalled()
  status.value = { ...status.value, status: 'waiting' }
  await nextTick()
  await vi.advanceTimersByTimeAsync(2000)
  expect(refresh).toHaveBeenCalledTimes(1)
  status.value = { ...status.value, status: 'failed' }
  await nextTick()
  await vi.advanceTimersByTimeAsync(10000)
  expect(refresh).toHaveBeenCalledTimes(1)
  status.value = { ...status.value, status: 'waiting' }
  await nextTick()
  await vi.advanceTimersByTimeAsync(2000)
  expect(refresh).toHaveBeenCalledTimes(2)
})
