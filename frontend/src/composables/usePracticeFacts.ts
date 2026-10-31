import { computed, onMounted, ref, shallowRef, watch } from 'vue'
import { api } from '../api'
import { takeTransfers } from '../takeTransfers'
import type { PracticeFacts, StudiedPage, StudiedPhrase } from '../factsTypes'
import { usePageReturn } from './usePageReturn'
import { usePagedList } from './usePagedList'

export function usePracticeFacts(message: () => string) {
  const facts = shallowRef<PracticeFacts | null>(null)
  const error = ref('')
  const listing = ref(false)
  const studied = usePagedList<StudiedPhrase, StudiedPage>(
    () => `/practice/studied?learning_date=${facts.value?.learning_date ?? ''}`,
    5,
  )
  // The completion message belongs to the learning day on which it appeared.
  const messageDay = ref<string | null>(null)
  const currentMessage = computed(() =>
    !facts.value || messageDay.value === null || messageDay.value === facts.value.learning_date
      ? message()
      : '',
  )
  let sequence = 0

  async function reload() {
    const request = ++sequence
    try {
      const next = await api<PracticeFacts>('/practice/facts')
      if (request !== sequence) return
      facts.value = next
      messageDay.value ??= next.learning_date
      error.value = ''
      if (listing.value) await studied.reload()
    } catch (cause) {
      if (request === sequence) error.value = String(cause)
    }
  }

  async function toggleList() {
    listing.value = !listing.value
    // Fresh facts fix the learning date before the list is fetched for it.
    if (listing.value) await reload()
  }

  watch(message, () => {
    messageDay.value = null
    void reload()
  })
  // A finished background upload turns a pending take into an available recording.
  watch(takeTransfers.pendingTakes, () => void reload())
  usePageReturn(() => void reload())
  onMounted(() => void reload())

  return {
    facts,
    error,
    listing,
    studied,
    currentMessage,
    uploadingTakes: takeTransfers.uploadingTakes,
    reload,
    toggleList,
  }
}
