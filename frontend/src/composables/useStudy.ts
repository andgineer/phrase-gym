import { ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'
import type { Card, HistoryItem, Study } from '../studyTypes'

type Completion = components['schemas']['StudyCompletion']

export function useStudy() {
  const current = ref<Study | null>(null)
  const history = ref<HistoryItem[]>([])
  const loading = ref(false)
  const completing = ref(false)
  const error = ref('')
  const rated = ref<string | null>(null)

  async function load() {
    loading.value = true
    error.value = ''
    try {
      ;[current.value, history.value] = await Promise.all([
        api<Study>('/study/current', { method: 'POST' }),
        api<HistoryItem[]>('/study/history'),
      ])
      rated.value = null
    } catch (cause) {
      error.value = String(cause)
    } finally {
      loading.value = false
    }
  }

  async function loadHistory() {
    history.value = await api<HistoryItem[]>('/study/history')
    const card = current.value?.card
    if (card) card.recordings = history.value.find((item) => item.id === card.id)?.recordings ?? []
  }

  async function reveal(attemptId: string, responseMode: 'spoken' | 'silent') {
    try {
      const card = await api<Card>(`/attempts/${attemptId}/reveal`, {
        method: 'POST',
        body: JSON.stringify({ response_mode: responseMode }),
      })
      if (current.value?.card?.attempt_id === attemptId) current.value.card = card
    } catch (cause) {
      error.value = String(cause)
    }
  }

  async function rate(attemptId: string, rating: string, responseMode: 'spoken' | 'silent') {
    if (completing.value) return
    completing.value = true
    error.value = ''
    try {
      await api<Completion>(`/attempts/${attemptId}/review`, {
        method: 'POST',
        body: JSON.stringify({
          operation_id: crypto.randomUUID(),
          rating,
          response_mode: responseMode,
        }),
      })
      rated.value = rating
      await loadHistory()
    } catch (cause) {
      error.value = String(cause)
    } finally {
      completing.value = false
    }
  }

  async function complete(attemptId: string, beforeComplete: () => Promise<void>) {
    if (completing.value) return
    completing.value = true
    error.value = ''
    try {
      await beforeComplete()
      await api<Completion>(`/attempts/${attemptId}/complete`, {
        method: 'POST',
        body: JSON.stringify({ operation_id: crypto.randomUUID() }),
      })
      await load()
    } catch (cause) {
      error.value = String(cause)
    } finally {
      completing.value = false
    }
  }

  return {
    current,
    history,
    loading,
    completing,
    error,
    rated,
    load,
    loadHistory,
    reveal,
    rate,
    complete,
  }
}
