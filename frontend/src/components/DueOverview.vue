<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { api } from '../api'
import type { Language } from '../api'
import type { components } from '../api.generated'
import type { PracticeMessages } from '../practiceMessages'
const props = defineProps<{ language: Language; lessonId?: string; pt: PracticeMessages }>()
const summary = ref<components['schemas']['DueSummary'] | null>(null)
const error = ref('')
let generation = 0
const nextDue = computed(() =>
  summary.value?.next_due_at ? new Date(summary.value.next_due_at).toLocaleString() : '',
)
async function load() {
  const token = ++generation
  summary.value = null
  error.value = ''
  try {
    const result = await api<components['schemas']['DueSummary']>(
      `/sessions/readiness?language=${props.language}${props.lessonId ? `&lesson_id=${props.lessonId}` : ''}`,
    )
    if (token === generation) summary.value = result
  } catch (cause) {
    if (token === generation) error.value = String(cause)
  }
}
watch([() => props.language, () => props.lessonId], load, { immediate: true })
</script>
<template>
  <div class="due-overview">
    <p v-if="summary">
      {{ pt.due }}: {{ summary.due_count }} · {{ pt.newTargets }}: {{ summary.new_count }} ·
      {{ pt.preparing }}: {{ summary.preparing_count }}
    </p>
    <p v-if="summary && !summary.due_count && !summary.new_count">{{ pt.noDue }}</p>
    <p v-if="nextDue">{{ pt.nextDue }}: {{ nextDue }}</p>
    <p v-if="error" class="error" role="alert">
      {{ error }} <button @click="load">{{ pt.retry }}</button>
    </p>
  </div>
</template>
