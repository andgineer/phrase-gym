import { onMounted, onUnmounted } from 'vue'

const refreshers = new Set<() => void>()

function returned() {
  if (document.visibilityState === 'visible') for (const refresh of refreshers) refresh()
}

function listen(add: boolean) {
  const method = add ? 'addEventListener' : 'removeEventListener'
  document[method]('visibilitychange', returned)
  window[method]('focus', returned)
  window[method]('pageshow', returned)
}

// A sleeping page can wake on a new learning day, so date-bound reads refresh on return.
export function usePageReturn(refresh: () => void) {
  onMounted(() => {
    if (!refreshers.size) listen(true)
    refreshers.add(refresh)
  })
  onUnmounted(() => {
    refreshers.delete(refresh)
    if (!refreshers.size) listen(false)
  })
}
