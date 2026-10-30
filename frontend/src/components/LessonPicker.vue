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
  <div class="lesson-picker">
    <label for="lesson-search">Search lessons</label>
    <input id="lesson-search" v-model="query" type="search" placeholder="Lesson or source title" />
    <label for="lesson-choice">Lesson</label>
    <select id="lesson-choice" v-model="selected">
      <option value="">All lessons</option>
      <option v-for="lesson in options" :key="lesson.id" :value="lesson.id">
        {{ lesson.title }} · {{ lesson.source_label }}
      </option>
    </select>
    <p v-if="counts" class="muted" data-testid="lesson-counts">
      {{ counts.due_count }} due · {{ counts.new_count }} new · {{ counts.phrase_count }} phrases
    </p>
    <p v-if="query.trim() && lessons.loaded.value && !lessons.items.value.length" class="muted">
      No matches
    </p>
    <button v-if="lessons.nextOffset.value !== null" class="quiet" @click="lessons.more">
      More lessons
    </button>
    <p v-if="lessons.error.value" role="alert" class="error">{{ lessons.error.value }}</p>
  </div>
</template>
