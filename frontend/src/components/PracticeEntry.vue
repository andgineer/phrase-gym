<script setup lang="ts">
import { ref } from 'vue'
import type { Recording } from '../studyTypes'
import LessonPicker from './LessonPicker.vue'
import LibraryPanel from './LibraryPanel.vue'

defineProps<{
  message: string
  error: string
  busy: boolean
  canRetryExtra: boolean
  play: (url: string) => Promise<void>
  compare: (recording: Recording, referenceUrl: string) => Promise<void>
  stopPlayback: () => void
}>()
const emit = defineEmits<{
  daily: []
  extra: []
  retryExtra: []
  practice: [phraseId: string]
}>()
const lessonId = defineModel<string>('lessonId', { default: '' })
const detailsPhrase = defineModel<string | null>('detailsPhrase', { default: null })
const picker = ref<InstanceType<typeof LessonPicker> | null>(null)
</script>

<template>
  <section v-show="!detailsPhrase" class="card session-entry">
    <LessonPicker ref="picker" v-model="lessonId" />
    <div class="actions">
      <button class="primary" :disabled="busy" @click="emit('daily')">Daily</button>
      <button :disabled="busy" @click="emit('extra')">Extra</button>
    </div>
    <p v-if="message" role="status" class="entry-message">{{ message }}</p>
  </section>
  <p v-if="error" role="alert" class="error">
    {{ error }}
    <button v-if="canRetryExtra" :disabled="busy" @click="emit('retryExtra')">Retry</button>
  </p>
  <LibraryPanel
    :lesson-id="lessonId"
    :disabled="busy"
    v-model:opened="detailsPhrase"
    :play="play"
    :compare="compare"
    :stop-playback="stopPlayback"
    @changed="picker?.reload()"
    @practice="emit('practice', $event)"
  />
</template>
