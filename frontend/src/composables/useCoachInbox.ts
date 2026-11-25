import { onMounted, ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'
import { usePageReturn } from './usePageReturn'

export type CoachOutcome = components['schemas']['CoachOutcome']
type CoachInbox = components['schemas']['CoachInbox']
type CoachSubmission = components['schemas']['CoachSubmission']

// Results are read when Coach opens or the page returns; Coach work is never polled.
export function useCoachInbox() {
  const items = ref<CoachOutcome[]>([])
  const error = ref('')
  const busy = ref<string | null>(null)
  let sequence = 0

  async function load() {
    const request = ++sequence
    try {
      const inbox = await api<CoachInbox>('/coach/inbox')
      if (request !== sequence) return
      items.value = inbox.items
      error.value = ''
    } catch (cause) {
      if (request === sequence) error.value = String(cause)
    }
  }

  async function act(requestId: string, action: 'dismiss' | 'retry') {
    if (busy.value) return
    busy.value = requestId
    try {
      await api<CoachSubmission | null>(
        `/coach/requests/${encodeURIComponent(requestId)}/${action}`,
        { method: 'POST' },
      )
      items.value = items.value.filter((item) => item.request_id !== requestId)
      error.value = ''
    } catch (cause) {
      error.value = cause instanceof Error ? cause.message : String(cause)
    } finally {
      busy.value = null
    }
  }

  usePageReturn(() => void load())
  onMounted(() => void load())
  return {
    items,
    error,
    busy,
    load,
    dismiss: (requestId: string) => act(requestId, 'dismiss'),
    retry: (requestId: string) => act(requestId, 'retry'),
  }
}
