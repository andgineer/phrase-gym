<script setup lang="ts">
import LessonPicker from './LessonPicker.vue'
import PracticeFacts from './PracticeFacts.vue'

defineProps<{
  message: string
  error: string
  busy: boolean
  canRetryExtra: boolean
}>()
const emit = defineEmits<{
  daily: []
  extra: []
  retryExtra: []
}>()
const lessonId = defineModel<string>('lessonId', { default: '' })
</script>

<template>
  <section class="card session-entry">
    <div class="actions">
      <button class="primary" :disabled="busy" @click="emit('daily')">Daily</button>
      <button :disabled="busy" @click="emit('extra')">Extra</button>
    </div>
    <LessonPicker v-model="lessonId" />
  </section>
  <p v-if="error" role="alert" class="error">
    {{ error }}
    <button v-if="canRetryExtra" :disabled="busy" @click="emit('retryExtra')">Retry</button>
  </p>
  <PracticeFacts :message="message" />
</template>
