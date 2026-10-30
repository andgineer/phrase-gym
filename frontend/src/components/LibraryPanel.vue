<script setup lang="ts">
import { ref } from 'vue'
import type { DeletionResult, DeletionTarget } from '../libraryTypes'
import type { Recording } from '../studyTypes'
import DeleteDialog from './DeleteDialog.vue'
import LessonPicker from './LessonPicker.vue'
import PhraseDetails from './PhraseDetails.vue'
import PhraseFinder from './PhraseFinder.vue'

const props = defineProps<{
  disabled: boolean
  play: (url: string) => Promise<void>
  compare: (recording: Recording, referenceUrl: string) => Promise<void>
  stopPlayback: () => void
}>()
const emit = defineEmits<{ practise: [phraseId: string] }>()
const lessonId = ref('')
const query = ref('')
const opened = ref<string | null>(null)
const pendingDelete = ref<DeletionTarget | null>(null)
const notice = ref('')
const lessons = ref<InstanceType<typeof LessonPicker> | null>(null)
const finder = ref<InstanceType<typeof PhraseFinder> | null>(null)

function open(phraseId: string) {
  notice.value = ''
  opened.value = phraseId
}

async function deleted(result: DeletionResult) {
  pendingDelete.value = null
  props.stopPlayback()
  opened.value = null
  notice.value = `Deleted “${result.deleted_text}”.`
  await Promise.all([finder.value?.reload(), lessons.value?.reload()])
}
</script>

<template>
  <PhraseDetails
    v-if="opened"
    :key="opened"
    :phrase-id="opened"
    :disabled="disabled"
    :play="play"
    :compare="compare"
    @back="opened = null"
    @practise="emit('practise', $event)"
    @remove="pendingDelete = $event"
  />
  <section v-show="!opened" class="card library">
    <h2>Library</h2>
    <p v-if="notice" role="status" class="success">{{ notice }}</p>
    <LessonPicker ref="lessons" v-model="lessonId" />
    <PhraseFinder
      ref="finder"
      v-model:query="query"
      :lesson-id="lessonId"
      :disabled="disabled"
      @open="open"
      @remove="pendingDelete = $event"
    />
  </section>
  <DeleteDialog :target="pendingDelete" @cancel="pendingDelete = null" @deleted="deleted" />
</template>
