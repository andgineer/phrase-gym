import { ref, shallowRef } from 'vue'
import { api } from '../api'

type Page<T> = { items: T[]; next_offset: number | null }
// The server's page limit.
const largestPage = 100

export function usePagedList<T, P extends Page<T> = Page<T>>(path: () => string, pageSize = 20) {
  const items = shallowRef<T[]>([])
  const page = shallowRef<P | null>(null)
  const nextOffset = ref<number | null>(null)
  const loading = ref(false)
  const loaded = ref(false)
  const error = ref('')
  let sequence = 0

  async function fetchItems(offset: number, wanted: number) {
    const request = ++sequence
    const base = path()
    const separator = base.includes('?') ? '&' : '?'
    loading.value = true
    error.value = ''
    try {
      let fetched: T[] = []
      let next: P
      let start = offset
      do {
        const limit = Math.min(wanted - fetched.length, largestPage)
        next = await api<P>(`${base}${separator}offset=${start}&limit=${limit}`)
        // A slower response for an earlier query must not replace newer results.
        if (request !== sequence) return
        fetched = [...fetched, ...next.items]
        start = next.next_offset ?? start
      } while (fetched.length < wanted && next.next_offset !== null)
      items.value = offset ? [...items.value, ...fetched] : fetched
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
    reload: () => fetchItems(0, pageSize),
    // Reloads every item already shown, so a chosen later item keeps fresh data.
    refresh: () => fetchItems(0, Math.max(items.value.length, pageSize)),
    more: () =>
      nextOffset.value === null ? Promise.resolve() : fetchItems(nextOffset.value, pageSize),
  }
}
