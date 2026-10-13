import { onMounted, onUnmounted } from 'vue'
import { connectionRetrying, serverUnavailable } from '../api'

export function useServerRetry(recover: () => Promise<void>, refresh: () => Promise<unknown>) {
  async function retry() {
    if (connectionRetrying.value) return
    connectionRetrying.value = true
    serverUnavailable.value = false
    try {
      await recover()
    } catch {
      serverUnavailable.value = true
    } finally {
      connectionRetrying.value = false
    }
  }
  function unavailable() {
    serverUnavailable.value = true
  }
  function resume() {
    if (!document.hidden) {
      if (serverUnavailable.value) void retry()
      else void refresh()
    }
  }
  onMounted(() => {
    window.addEventListener('offline', unavailable)
    window.addEventListener('online', resume)
    document.addEventListener('visibilitychange', resume)
  })
  onUnmounted(() => {
    window.removeEventListener('offline', unavailable)
    window.removeEventListener('online', resume)
    document.removeEventListener('visibilitychange', resume)
  })
  return { retry, connectionRetrying, serverUnavailable }
}
