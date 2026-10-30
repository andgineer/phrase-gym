<script setup lang="ts">
import { ref } from 'vue'
import type { DeletionResult, DeletionTarget } from '../libraryTypes'
import type { Recording } from '../studyTypes'
import DeleteDialog from './DeleteDialog.vue'
import PhraseDetails from './PhraseDetails.vue'
import PhraseFinder from './PhraseFinder.vue'

const props = defineProps<{
  lessonId: string
  disabled: boolean
  play: (url: string) => Promise<void>
  compare: (recording: Recording, referenceUrl: string) => Promise<void>
  stopPlayback: () => void
}>()
const emit = defineEmits<{ changed: []; practice: [phraseId: string] }>()
const opened = defineModel<string | null>('opened', { default: null })
const query = ref('')
const pendingDelete = ref<DeletionTarget | null>(null)
const notice = ref('')
const finder = ref<InstanceType<typeof PhraseFinder> | null>(null)

function open(phraseId: string | null) {
  notice.value = ''
  opened.value = phraseId
}

async function deleted(result: DeletionResult) {
  pendingDelete.value = null
  props.stopPlayback()
  open(null)
  notice.value = `Deleted “${result.deleted_text}”.`
  emit('changed')
  await finder.value?.reload()
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
    @back="open(null)"
    @remove="pendingDelete = $event"
    @practice="emit('practice', $event)"
  />
  <section v-show="!opened" class="card library">
    <h2>Library</h2>
    <p v-if="notice" role="status" class="success">{{ notice }}</p>
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
