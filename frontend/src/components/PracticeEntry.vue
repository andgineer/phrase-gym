<script setup lang="ts">
import { toRef, watch } from 'vue'
import LessonPicker from './LessonPicker.vue'
import PracticeFacts from './PracticeFacts.vue'
import { usePracticeToday } from '../composables/usePracticeToday'
import type { LearningLanguage } from '../languages'

const props = defineProps<{
  language: LearningLanguage
  message: string
  error: string
  busy: boolean
  canRetryExtra: boolean
}>()
const emit = defineEmits<{ daily: []; extra: []; retryExtra: [] }>()
const lessonId = defineModel<string>('lessonId', { default: '' })
const { today, error: countsError, refresh } = usePracticeToday(toRef(lessonId), props.language)
watch(
  () => props.busy,
  (busy) => {
    if (!busy) void refresh()
  },
)
</script>

<template>
  <section class="card session-entry">
    <LessonPicker v-model="lessonId" :language="language" compact />
    <div class="start-summary">
      <div v-if="today && today.due_count + today.new_count" class="waiting-counts">
        <span
          ><strong>{{ today.due_count }}</strong> due</span
        >
        <span
          ><strong>{{ today.new_count }}</strong> new</span
        >
      </div>
      <p v-else-if="today && today.message !== 'Add your first phrase with Coach to begin.'">
        {{ today.message }}
      </p>
    </div>
    <div class="start-actions">
      <button
        :class="{ primary: today && today.due_count + today.new_count > 0 }"
        :disabled="busy || !today || today.due_count + today.new_count === 0"
        @click="emit('daily')"
      >
        Start
      </button>
      <button
        v-if="today && !today.due_count && !today.new_count && today.extra_available"
        class="primary"
        :disabled="busy"
        @click="emit('extra')"
      >
        Extra
      </button>
    </div>
  </section>
  <p v-if="countsError" role="alert" class="error">
    {{ countsError }} <button @click="refresh">Retry</button>
  </p>
  <p v-if="error" role="alert" class="error">
    {{ error }}
    <button v-if="canRetryExtra" :disabled="busy" @click="emit('retryExtra')">Retry</button>
  </p>
  <PracticeFacts :language="language" :message="message.includes('was saved') ? message : ''" />
</template>
