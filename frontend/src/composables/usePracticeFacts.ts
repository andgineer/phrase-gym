import { onMounted, ref, shallowRef, watch } from 'vue'
import { api } from '../api'
import { takeTransfers } from '../takeTransfers'
import type { PracticeFacts, StudiedPage, StudiedPhrase } from '../factsTypes'
import { usePagedList } from './usePagedList'

export function usePracticeFacts() {
  const facts = shallowRef<PracticeFacts | null>(null)
  const error = ref('')
  const listing = ref(false)
  const studied = usePagedList<StudiedPhrase, StudiedPage>(
    () => `/practice/studied?learning_date=${facts.value?.learning_date ?? ''}`,
    5,
  )
  let sequence = 0

  async function reload() {
    const request = ++sequence
    try {
      const next = await api<PracticeFacts>('/practice/facts')
      if (request !== sequence) return
      facts.value = next
      error.value = ''
      if (listing.value) await studied.reload()
    } catch (cause) {
      if (request === sequence) error.value = String(cause)
    }
  }

  function toggleList() {
    listing.value = !listing.value
    if (listing.value) void studied.reload()
  }

  // A finished background upload turns a pending take into an available recording.
  watch(takeTransfers.pendingTakes, () => void reload())
  onMounted(() => void reload())

  return {
    facts,
    error,
    listing,
    studied,
    pendingTakes: takeTransfers.pendingTakes,
    reload,
    toggleList,
  }
}
