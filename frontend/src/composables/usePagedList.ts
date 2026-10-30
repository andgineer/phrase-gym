import { ref, shallowRef } from 'vue'
import { api } from '../api'

type Page<T> = { items: T[]; next_offset: number | null }

export function usePagedList<T, P extends Page<T> = Page<T>>(path: () => string, pageSize = 20) {
  const items = shallowRef<T[]>([])
  const page = shallowRef<P | null>(null)
  const nextOffset = ref<number | null>(null)
  const loading = ref(false)
  const loaded = ref(false)
  const error = ref('')
  let sequence = 0

  async function fetchPage(offset: number) {
    const request = ++sequence
    const base = path()
    loading.value = true
    error.value = ''
    try {
      const separator = base.includes('?') ? '&' : '?'
      const next = await api<P>(`${base}${separator}offset=${offset}&limit=${pageSize}`)
      // A slower response for an earlier query must not replace newer results.
      if (request !== sequence) return
      items.value = offset ? [...items.value, ...next.items] : next.items
      page.value = next
      nextOffset.value = next.next_offset
      loaded.value = true
    } catch (cause) {
      if (request === sequence) error.value = String(cause)
    } finally {
      if (request === sequence) loading.value = false
    }
  }

  return {
    items,
    page,
    nextOffset,
    loading,
    loaded,
    error,
    reload: () => fetchPage(0),
    more: () => (nextOffset.value === null ? Promise.resolve() : fetchPage(nextOffset.value)),
  }
}
