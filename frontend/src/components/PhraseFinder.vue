<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { usePagedList } from '../composables/usePagedList'
import LibraryCoach from './LibraryCoach.vue'
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
const queryPending = ref(false)
let debounce: number | undefined

watch(query, () => {
  queryPending.value = true
  window.clearTimeout(debounce)
  const current = query.value
  debounce = window.setTimeout(async () => {
    await phrases.reload()
    if (query.value === current) queryPending.value = false
  }, 200)
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
    <div class="phrase-search">
      <span aria-hidden="true">⌕</span>
      <input
        v-model="query"
        type="search"
        aria-label="Search phrases or concepts"
        placeholder="Search phrases or concepts"
      />
    </div>
    <slot v-if="!searching" name="lesson" />
    <p v-if="searching" class="muted">Searching all lessons.</p>
    <ul
      v-if="!queryPending && phrases.items.value.length"
      class="phrase-list library-results"
      aria-label="Phrases"
    >
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
    <p
      v-else-if="
        (searching || lessonId) &&
        !queryPending &&
        phrases.loaded.value &&
        !phrases.loading.value &&
        !phrases.error.value
      "
      class="muted"
      role="status"
    >
      {{ searching ? 'Not in your Library yet.' : 'No phrases in this lesson' }}
    </p>
    <LibraryCoach
      v-if="
        searching &&
        !queryPending &&
        phrases.loaded.value &&
        !phrases.loading.value &&
        !phrases.error.value
      "
      :text="query.trim()"
      :has-results="phrases.items.value.length > 0"
    />
    <button v-if="!queryPending && phrases.nextOffset.value !== null" @click="phrases.more">
      More
    </button>
    <p v-if="phrases.error.value" role="alert" class="error">{{ phrases.error.value }}</p>
  </div>
</template>
