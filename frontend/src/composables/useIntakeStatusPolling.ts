import { onUnmounted, watch, type Ref } from 'vue'
import { connectionRetrying, serverUnavailable } from '../api'
import type { components } from '../api.generated'

type Status = components['schemas']['LessonStatus']
export function useIntakeStatusPolling(
  generation: () => string | undefined,
  status: Ref<Status | undefined>,
  refresh: () => Promise<void>,
) {
  let timer: ReturnType<typeof setTimeout> | undefined
  let epoch = 0
  watch([generation, () => status.value?.status], () => {
    clearTimeout(timer)
    const token = ++epoch
    if (
      !generation() ||
      ['published', 'failed', 'cancelled', 'superseded'].includes(status.value?.status ?? '')
    )
      return
    let delay = 2000
    async function tick() {
      if (token !== epoch) return
      if (!serverUnavailable.value && !connectionRetrying.value) await refresh()
      if (
        token !== epoch ||
        ['published', 'failed', 'cancelled', 'superseded'].includes(status.value?.status ?? '')
      )
        return
      delay = Math.min(delay * 2, 10000)
      timer = setTimeout(tick, delay)
    }
    timer = setTimeout(tick, delay)
  })
  onUnmounted(() => {
    epoch++
    clearTimeout(timer)
  })
}
