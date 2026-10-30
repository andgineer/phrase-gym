import { ref } from 'vue'
import { ApiError, api } from '../api'
import type { components } from '../api.generated'
import type { Card, RecordingPage, Study } from '../studyTypes'

type Completion = components['schemas']['StudyCompletion']

export function useStudy() {
  const current = ref<Study | null>(null)
  const loading = ref(false)
  const completing = ref(false)
  const error = ref('')
  const rated = ref<string | null>(null)

  async function load() {
    loading.value = true
    error.value = ''
    try {
      current.value = await api<Study>('/study/current', { method: 'POST' })
      rated.value = null
    } catch (cause) {
      error.value = String(cause)
    } finally {
      loading.value = false
    }
  }

  async function refreshRecordings() {
    const card = current.value?.card
    if (!card) return
    try {
      const page = await api<RecordingPage>(`/phrases/${card.id}/recordings?limit=100`)
      if (current.value?.card?.attempt_id === card.attempt_id)
        current.value.card.recordings = [...page.items].reverse()
    } catch (cause) {
      // A deleted phrase has no recordings left to show on its old card.
      if (!(cause instanceof ApiError && cause.status === 404)) error.value = String(cause)
    }
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
      await refreshRecordings()
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 409) {
        await load()
        error.value = 'This review is no longer independent. It was skipped without a grade.'
      } else error.value = String(cause)
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
    loading,
    completing,
    error,
    rated,
    load,
    refreshRecordings,
    reveal,
    rate,
    complete,
  }
}
