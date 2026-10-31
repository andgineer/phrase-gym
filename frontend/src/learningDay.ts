import { computed, ref, watch } from 'vue'

// The last learning date reported by the server; shared so every date-bound view resets together.
const learningDay = ref<string | null>(null)

export function observeLearningDay(day: string) {
  const changed = learningDay.value !== null && learningDay.value !== day
  learningDay.value = day
  return changed
}

export function onLearningDayChange(refresh: () => void) {
  watch(learningDay, (day, previous) => {
    if (previous !== null && day !== previous) refresh()
  })
}

// A message belongs to the learning date on which it appeared, even after the entry remounts.
export function useDatedMessage(message: () => string) {
  const created = ref<string | null>(null)
  watch(message, (text) => (created.value = text ? learningDay.value : null), { immediate: true })
  return computed(() =>
    created.value === null || created.value === learningDay.value ? message() : '',
  )
}
