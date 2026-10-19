<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { api } from '../api'
import type { Language } from '../api'
import type { components } from '../../tests/rust-contract/api.generated'
import { onboardingMessages } from '../onboardingMessages'
import type { PracticeMessages } from '../practiceMessages'
const props = defineProps<{
  language: Language
  lessonId?: string
  pt: PracticeMessages
  locale: Language
}>()
const emit = defineEmits<{ eligible: [boolean] }>()
const summary = ref<components['schemas']['DueSummary'] | null>(null)
const error = ref('')
let generation = 0
const nextDue = computed(() =>
  summary.value?.next_due_at ? new Date(summary.value.next_due_at).toLocaleString() : '',
)
async function load() {
  const token = ++generation
  emit('eligible', false)
  summary.value = null
  error.value = ''
  try {
    const result = await api<components['schemas']['DueSummary']>(
      `/sessions/readiness?language=${props.language}${props.lessonId ? `&lesson_id=${props.lessonId}` : ''}`,
    )
    if (token === generation) {
      summary.value = result
      emit('eligible', result.due_count + result.new_count > 0)
    }
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
      {{ onboardingMessages(locale).awaiting }}: {{ summary.preparing_count }}
    </p>
    <p v-if="summary?.reference_only_count">
      {{ onboardingMessages(locale).referenceCount }}: {{ summary.reference_only_count }}.
      {{ onboardingMessages(locale).referenceOnly }}
    </p>
    <p v-if="summary && !summary.due_count && !summary.new_count">{{ pt.noDue }}</p>
    <p v-if="nextDue">{{ pt.nextDue }}: {{ nextDue }}</p>
    <p v-if="error" class="error" role="alert">
      {{ error }} <button @click="load">{{ pt.retry }}</button>
    </p>
  </div>
</template>
