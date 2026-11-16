import { onUnmounted, ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'

type CoachSubmission = components['schemas']['CoachSubmission']
const confirmations: Record<CoachSubmission['outcome'], string> = {
  saved: 'Saved for Coach.',
  waiting: 'Already waiting for Coach.',
  in_library: 'Already in your Library.',
}

export const coachText = (text: string) => text.replace(/[\r\n]+/g, ' ').trim()

export function useCoachSubmission() {
  const sending = ref(false)
  const confirmation = ref('')
  const error = ref('')
  let hideConfirmation: number | undefined

  function reset() {
    window.clearTimeout(hideConfirmation)
    confirmation.value = ''
    error.value = ''
  }

  async function submit(text: string) {
    const typed = coachText(text)
    if (!typed || sending.value) return false
    sending.value = true
    reset()
    try {
      const saved = await api<CoachSubmission>('/coach/requests', {
        method: 'POST',
        body: JSON.stringify({ text: typed }),
      })
      confirmation.value = confirmations[saved.outcome]
      hideConfirmation = window.setTimeout(() => (confirmation.value = ''), 4000)
      return true
    } catch (cause) {
      error.value = cause instanceof Error ? cause.message : String(cause)
      return false
    } finally {
      sending.value = false
    }
  }

  onUnmounted(reset)
  return { sending, confirmation, error, submit, reset }
}
