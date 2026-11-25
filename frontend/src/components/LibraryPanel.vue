<script setup lang="ts">
import { ref } from 'vue'
import type { DeletionResult, DeletionTarget } from '../libraryTypes'
import LessonPicker from './LessonPicker.vue'
import DeleteDialog from './DeleteDialog.vue'
import PhraseDetails from './PhraseDetails.vue'
import PhraseFinder from './PhraseFinder.vue'
import type { LearningLanguage } from '../languages'

const props = defineProps<{
  language: LearningLanguage
  disabled: boolean
  play: (url: string) => Promise<void>
  stopPlayback: () => void
}>()
const emit = defineEmits<{ practice: [phraseId: string] }>()
const lessonId = defineModel<string>('lessonId', { default: '' })
const picker = ref<InstanceType<typeof LessonPicker> | null>(null)
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
  void picker.value?.reload()
  await finder.value?.reload()
}
</script>

<template>
  <PhraseDetails
    v-if="opened"
    :key="opened"
    :phrase-id="opened"
    :language="language"
    :disabled="disabled"
    :play="play"
    @back="open(null)"
    @remove="pendingDelete = $event"
    @practice="emit('practice', $event)"
  />
  <section v-show="!opened" class="card library" aria-label="Library">
    <p v-if="notice" role="status" class="success">{{ notice }}</p>
    <PhraseFinder
      ref="finder"
      v-model:query="query"
      :language="language"
      :lesson-id="lessonId"
      :disabled="disabled"
      @open="open"
    >
      <template #lesson
        ><LessonPicker ref="picker" v-model="lessonId" :language="language" compact
      /></template>
    </PhraseFinder>
  </section>
  <DeleteDialog :target="pendingDelete" @cancel="pendingDelete = null" @deleted="deleted" />
</template>
