<script setup lang="ts">
import { ref } from 'vue'
import type { Recording } from '../studyTypes'
import LessonPicker from './LessonPicker.vue'
import LibraryPanel from './LibraryPanel.vue'

defineProps<{
  message: string
  error: string
  busy: boolean
  play: (url: string) => Promise<void>
  compare: (recording: Recording, referenceUrl: string) => Promise<void>
  stopPlayback: () => void
}>()
const emit = defineEmits<{ daily: [] }>()
const lessonId = defineModel<string>('lessonId', { default: '' })
const detailsOpen = ref(false)
const picker = ref<InstanceType<typeof LessonPicker> | null>(null)
</script>

<template>
  <section v-show="!detailsOpen" class="card session-entry">
    <LessonPicker ref="picker" v-model="lessonId" />
    <div class="actions">
      <button class="primary" :disabled="busy" @click="emit('daily')">Daily</button>
    </div>
    <p v-if="message" role="status" class="entry-message">{{ message }}</p>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
  </section>
  <LibraryPanel
    :lesson-id="lessonId"
    :disabled="false"
    :play="play"
    :compare="compare"
    :stop-playback="stopPlayback"
    @details="detailsOpen = $event"
    @changed="picker?.reload()"
  />
</template>
