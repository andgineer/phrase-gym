import { onUnmounted, ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'
import type { LearningLanguage } from '../languages'

type CoachSubmission = components['schemas']['CoachSubmission']
type CoachRequest = components['schemas']['CoachRequest']
const confirmations: Record<CoachSubmission['outcome'], string> = {
  saved: 'Saved for Coach.',
  waiting: 'Already waiting for Coach.',
  in_library: 'Already in your Library.',
  rejected: 'Already checked: Coach could not add this. Its reason is in Coach.',
}

export const coachText = (text: string) => text.replace(/[\r\n]+/g, ' ').trim()

// A send captures the language of the page in use; a language switch remounts the page.
export function useCoachSubmission(pageLanguage: LearningLanguage) {
  const sending = ref(false)
  const confirmation = ref('')
  const error = ref('')
  let hideConfirmation: number | undefined

  function reset() {
    window.clearTimeout(hideConfirmation)
    confirmation.value = ''
    error.value = ''
  }

  async function submit(text: string): Promise<CoachSubmission['outcome'] | null> {
    const typed = coachText(text)
    if (!typed || sending.value) return null
    sending.value = true
    reset()
    try {
      const saved = await api<CoachSubmission>('/coach/requests', {
        method: 'POST',
        body: JSON.stringify({
          text: typed,
          learning_language: pageLanguage,
        } satisfies CoachRequest),
      })
      confirmation.value = confirmations[saved.outcome]
      hideConfirmation = window.setTimeout(() => (confirmation.value = ''), 4000)
      return saved.outcome
    } catch (cause) {
      error.value = cause instanceof Error ? cause.message : String(cause)
      return null
    } finally {
      sending.value = false
    }
  }

  onUnmounted(reset)
  return { sending, confirmation, error, submit, reset }
}
