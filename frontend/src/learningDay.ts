import { computed, ref, watch } from 'vue'

// The last learning date reported by the server; shared so every date-bound view resets together.
const learningDay = ref<string | null>(null)
const observations = ref(0)

export function observeLearningDay(day: string) {
  const changed = learningDay.value !== null && learningDay.value !== day
  learningDay.value = day
  observations.value++
  return changed
}

export function onLearningDayChange(refresh: () => void) {
  watch(learningDay, (day, previous) => {
    if (previous !== null && day !== previous) refresh()
  })
}

// The browser's known day may be stale when a message appears, so the next server date dates it.
// Every practice state is a new object, so a repeated identical message is dated afresh.
export function useDatedMessage(state: () => { message: string }) {
  const dated = ref<string | null>(null)
  watch(state, () => (dated.value = null))
  watch(observations, () => {
    if (state().message && dated.value === null) dated.value = learningDay.value
  })
  return computed(() =>
    dated.value === null || dated.value === learningDay.value ? state().message : '',
  )
}
