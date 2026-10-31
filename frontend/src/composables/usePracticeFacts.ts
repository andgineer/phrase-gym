import { onMounted, ref, shallowRef, watch } from 'vue'
import { api } from '../api'
import { takeTransfers } from '../takeTransfers'
import type { PracticeFacts, StudiedPage, StudiedPhrase } from '../factsTypes'
import { observeLearningDay } from '../learningDay'
import { usePageReturn } from './usePageReturn'
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

  // Counts are cheap to refresh; the paged list restarts only when asked or on a new day.
  async function reload(list = false) {
    const request = ++sequence
    try {
      const next = await api<PracticeFacts>('/practice/facts')
      if (request !== sequence) return
      const newDay = observeLearningDay(next.learning_date)
      facts.value = next
      error.value = ''
      if (listing.value && (list || newDay)) await studied.reload()
    } catch (cause) {
      if (request === sequence) error.value = String(cause)
    }
  }

  async function toggleList() {
    listing.value = !listing.value
    // Fresh facts fix the learning date before the list is fetched for it.
    if (listing.value) await reload(true)
  }

  // A finished background upload turns a pending take into an available recording.
  watch(takeTransfers.pendingTakes, () => void reload())
  usePageReturn(() => void reload())
  onMounted(() => void reload())

  return {
    facts,
    error,
    listing,
    studied,
    uploadingTakes: takeTransfers.uploadingTakes,
    refresh: () => reload(true),
    toggleList,
  }
}
