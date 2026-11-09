<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { usePagedList } from '../composables/usePagedList'
import type { PhrasePage, PhraseSummary } from '../libraryTypes'

const props = defineProps<{ lessonId: string; disabled: boolean }>()
const emit = defineEmits<{ open: [phraseId: string] }>()
const query = defineModel<string>('query', { default: '' })
const searching = computed(() => query.value.trim() !== '')
const phrases = usePagedList<PhraseSummary, PhrasePage>(() =>
  searching.value
    ? `/phrases?query=${encodeURIComponent(query.value.trim())}`
    : `/phrases${props.lessonId ? `?lesson_id=${encodeURIComponent(props.lessonId)}` : ''}`,
)
const empty = computed(() =>
  searching.value
    ? 'No matches'
    : props.lessonId
      ? 'No phrases in this lesson'
      : 'No phrases yet. Use Add material to import a package.',
)
let debounce: number | undefined

watch(query, () => {
  window.clearTimeout(debounce)
  debounce = window.setTimeout(() => void phrases.reload(), 200)
})
watch(
  () => props.lessonId,
  () => {
    if (!searching.value) void phrases.reload()
  },
)
onMounted(() => void phrases.reload())
onUnmounted(() => window.clearTimeout(debounce))
defineExpose({ reload: phrases.reload })
</script>

<template>
  <div class="phrase-finder">
    <input
      v-model="query"
      type="search"
      aria-label="Search phrases or concepts"
      placeholder="Search phrases or concepts"
    />
    <p v-if="searching" class="muted">Searching all lessons.</p>
    <ul v-if="phrases.items.value.length" class="phrase-list library-results" aria-label="Phrases">
      <li v-for="(phrase, index) in phrases.items.value" :key="phrase.id">
        <button
          class="phrase-summary"
          :aria-labelledby="`phrase-result-${index}`"
          :aria-describedby="`phrase-result-${index}-use phrase-result-${index}-lesson`"
          :disabled="disabled"
          @click="emit('open', phrase.id)"
        >
          <strong :id="`phrase-result-${index}`" lang="en">{{ phrase.text }}</strong>
          <span :id="`phrase-result-${index}-use`" class="purpose"
            >Used for: {{ phrase.usage_contexts.join(' · ') }}</span
          >
          <span :id="`phrase-result-${index}-lesson`" class="muted">{{ phrase.lesson_title }}</span>
        </button>
      </li>
    </ul>
    <p v-else-if="phrases.loaded.value && !phrases.loading.value" class="muted" role="status">
      {{ empty }}
    </p>
    <button v-if="phrases.nextOffset.value !== null" @click="phrases.more">More</button>
    <p v-if="phrases.error.value" role="alert" class="error">{{ phrases.error.value }}</p>
  </div>
</template>
