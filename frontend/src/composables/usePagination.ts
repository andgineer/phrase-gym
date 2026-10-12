import { ref } from 'vue'
import type { Ref } from 'vue'

export function usePagination<T>(options: {
  items: Ref<T[]>
  total: Ref<number>
  version: () => number
  fetchPage: () => Promise<{ items: T[]; total: number }>
  error: Ref<string>
}) {
  const loading = ref(false)
  function reset() {
    loading.value = false
  }
  async function loadMore() {
    if (loading.value) return
    const version = options.version()
    loading.value = true
    options.error.value = ''
    try {
      const page = await options.fetchPage()
      if (version === options.version()) {
        options.items.value.push(...page.items)
        options.total.value = page.total
      }
    } catch (cause) {
      if (version === options.version()) options.error.value = String(cause)
    } finally {
      if (version === options.version()) loading.value = false
    }
  }
  return { loading, loadMore, reset }
}
