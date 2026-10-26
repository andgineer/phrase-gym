import { ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'
import type { HistoryItem, Study } from '../studyTypes'

type Completion = components['schemas']['StudyCompletion']

export function useStudy() {
  const current = ref<Study | null>(null)
  const history = ref<HistoryItem[]>([])
  const loading = ref(false)
  const completing = ref(false)
  const error = ref('')

  async function load() {
    loading.value = true
    error.value = ''
    try {
      ;[current.value, history.value] = await Promise.all([
        api<Study>('/study/current', { method: 'POST' }),
        api<HistoryItem[]>('/study/history'),
      ])
    } catch (cause) {
      error.value = String(cause)
    } finally {
      loading.value = false
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

  return { current, history, loading, completing, error, load, complete }
}
