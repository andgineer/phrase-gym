<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { usePagedList } from '../composables/usePagedList'
import type { LessonChoice, LessonPage } from '../libraryTypes'

const selected = defineModel<string>({ default: '' })
const query = ref('')
const chosen = ref<LessonChoice | null>(null)
const lessons = usePagedList<LessonChoice, LessonPage>(
  () => `/lessons?query=${encodeURIComponent(query.value.trim())}`,
)
const options = computed(() =>
  chosen.value && !lessons.items.value.some((lesson) => lesson.id === chosen.value?.id)
    ? [chosen.value, ...lessons.items.value]
    : lessons.items.value,
)
// Only a freshly loaded lesson has current counts; a lesson hidden by the search may be stale.
const counts = computed(() =>
  selected.value
    ? lessons.items.value.find((lesson) => lesson.id === selected.value)
    : lessons.page.value?.all_lessons,
)
const empty = computed(() => lessons.page.value?.all_lessons.phrase_count === 0)
const summary = computed(() => {
  const value = counts.value
  if (!value) return ''
  if (!value.phrase_count) return 'No phrases in this lesson'
  if (value.due_count || value.new_count) return `${value.due_count} due · ${value.new_count} new`
  return value.next_due
    ? `Nothing waiting today · Next due: ${value.next_due}`
    : 'Nothing waiting today'
})
let debounce: number | undefined

watch(query, () => {
  window.clearTimeout(debounce)
  debounce = window.setTimeout(() => void lessons.reload(), 200)
})
watch(selected, (id) => {
  chosen.value = lessons.items.value.find((lesson) => lesson.id === id) ?? null
})
watch(lessons.items, (items) => {
  const fresh = items.find((lesson) => lesson.id === selected.value)
  if (fresh) chosen.value = fresh
})
onMounted(() => void lessons.reload())
onUnmounted(() => window.clearTimeout(debounce))
defineExpose({ reload: lessons.reload })
</script>

<template>
  <div v-if="empty" class="lesson-picker">
    <p>Add your first phrase batch</p>
    <p class="muted">Import 5–10 prepared phrases to begin.</p>
  </div>
  <div v-else class="lesson-picker">
    <label for="lesson-search">Search lessons</label>
    <input id="lesson-search" v-model="query" type="search" placeholder="Lesson or source title" />
    <label for="lesson-choice">Lesson to drill</label>
    <select id="lesson-choice" v-model="selected">
      <option value="">All lessons</option>
      <option v-for="lesson in options" :key="lesson.id" :value="lesson.id">
        {{ lesson.title }} · {{ lesson.source_label }}
      </option>
    </select>
    <p v-if="summary" class="muted" data-testid="lesson-counts">{{ summary }}</p>
    <p v-if="query.trim() && lessons.loaded.value && !lessons.items.value.length" class="muted">
      No matches
    </p>
    <button v-if="lessons.nextOffset.value !== null" class="quiet" @click="lessons.more">
      More lessons
    </button>
    <p v-if="lessons.error.value" role="alert" class="error">{{ lessons.error.value }}</p>
  </div>
</template>
