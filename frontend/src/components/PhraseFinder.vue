<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { usePagedList } from '../composables/usePagedList'
import type { DeletionTarget, PhrasePage, PhraseSummary } from '../libraryTypes'

const props = defineProps<{ lessonId: string; disabled: boolean }>()
const emit = defineEmits<{ open: [phraseId: string]; remove: [target: DeletionTarget] }>()
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
    <label for="phrase-search">Find a phrase</label>
    <input
      id="phrase-search"
      v-model="query"
      type="search"
      placeholder="Search phrases or concepts"
    />
    <p v-if="searching" class="muted">Searching all lessons.</p>
    <ul v-if="phrases.items.value.length" class="phrase-list library-results" aria-label="Phrases">
      <li v-for="phrase in phrases.items.value" :key="phrase.id">
        <div class="phrase-summary">
          <strong lang="en">{{ phrase.text }}</strong>
          <span class="purpose">Used for: {{ phrase.usage_contexts.join(' · ') }}</span>
          <span class="muted">{{ phrase.lesson_title }}</span>
        </div>
        <div class="row-actions">
          <button
            :aria-label="`Open details: ${phrase.text}`"
            :disabled="disabled"
            @click="emit('open', phrase.id)"
          >
            Open details
          </button>
          <button
            :aria-label="`Delete: ${phrase.text}`"
            :disabled="disabled"
            @click="emit('remove', { id: phrase.id, text: phrase.text })"
          >
            Delete
          </button>
        </div>
      </li>
    </ul>
    <p v-else-if="phrases.loaded.value && !phrases.loading.value" class="muted" role="status">
      {{ empty }}
    </p>
    <button v-if="phrases.nextOffset.value !== null" @click="phrases.more">More</button>
    <p v-if="phrases.error.value" role="alert" class="error">{{ phrases.error.value }}</p>
  </div>
</template>
