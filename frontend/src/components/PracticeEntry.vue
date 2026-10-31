<script setup lang="ts">
import { ref } from 'vue'
import type { Recording } from '../studyTypes'
import LessonPicker from './LessonPicker.vue'
import LibraryPanel from './LibraryPanel.vue'
import PracticeFacts from './PracticeFacts.vue'

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
const facts = ref<InstanceType<typeof PracticeFacts> | null>(null)

function libraryChanged() {
  void picker.value?.reload()
  void facts.value?.refresh()
}
</script>

<template>
  <PracticeFacts v-show="!detailsPhrase" ref="facts" :message="message" />
  <section v-show="!detailsPhrase" class="card session-entry">
    <LessonPicker ref="picker" v-model="lessonId" />
    <div class="actions">
      <button class="primary" :disabled="busy" @click="emit('daily')">Daily</button>
      <button :disabled="busy" @click="emit('extra')">Extra</button>
    </div>
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
    @changed="libraryChanged"
    @practice="emit('practice', $event)"
  />
</template>
