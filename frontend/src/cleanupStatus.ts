import { ref } from 'vue'
import { api } from './api'
import type { CleanupStatus } from './libraryTypes'

const status = ref<CleanupStatus>({ pending: 0, error: null })
const failure = ref('')

async function update(request: Promise<CleanupStatus>) {
  try {
    status.value = await request
    failure.value = ''
  } catch (cause) {
    failure.value = String(cause)
  }
}

export const fileCleanup = {
  status,
  failure,
  refresh: () => update(api<CleanupStatus>('/cleanup')),
  retry: () => update(api<CleanupStatus>('/cleanup/retry', { method: 'POST' })),
  set: (next: CleanupStatus) => (status.value = next),
}
