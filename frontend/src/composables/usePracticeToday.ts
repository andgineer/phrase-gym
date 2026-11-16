import { onMounted, ref, watch, type Ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'
import { observeLearningDay, onLearningDayChange } from '../learningDay'
import { usePageReturn } from './usePageReturn'

export function usePracticeToday(lessonId: Ref<string>) {
  const today = ref<components['schemas']['PracticeToday'] | null>(null)
  const error = ref('')
  let sequence = 0
  async function refresh() {
    const request = ++sequence
    today.value = null
    try {
      const result = await api<components['schemas']['PracticeToday']>(
        `/practice/today${lessonId.value ? `?lesson_id=${encodeURIComponent(lessonId.value)}` : ''}`,
      )
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
