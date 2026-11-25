import { onMounted, ref, watch, type Ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'
import { observeLearningDay, onLearningDayChange } from '../learningDay'
import type { LearningLanguage } from '../languages'
import { usePageReturn } from './usePageReturn'

// The entry remounts when its language changes, so the language is fixed for this instance.
export function usePracticeToday(lessonId: Ref<string>, language: LearningLanguage) {
  const today = ref<components['schemas']['PracticeToday'] | null>(null)
  const error = ref('')
  let sequence = 0
  async function refresh() {
    const request = ++sequence
    today.value = null
    try {
      const params = new URLSearchParams({ language })
      if (lessonId.value) params.set('lesson_id', lessonId.value)
      const result = await api<components['schemas']['PracticeToday']>(`/practice/today?${params}`)
      if (request !== sequence) return
      today.value = result
      error.value = ''
      observeLearningDay(result.learning_date)
    } catch (cause) {
      if (request === sequence) error.value = String(cause)
    }
  }
  watch(lessonId, refresh)
  onMounted(refresh)
  onLearningDayChange(() => void refresh())
  usePageReturn(() => void refresh())
  return { today, error, refresh }
}
