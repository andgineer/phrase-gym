<script setup lang="ts">
import PhraseExplanation from './PhraseExplanation.vue'
import AudioIcon from './AudioIcon.vue'
import { onMounted, ref } from 'vue'
import { api } from '../api'
import type { DeletionTarget, PhraseDetails } from '../libraryTypes'
import SavedHistory from './SavedHistory.vue'

const props = defineProps<{
  phraseId: string
  disabled: boolean
  play: (url: string) => Promise<void>
}>()
const emit = defineEmits<{ back: []; remove: [DeletionTarget]; practice: [phraseId: string] }>()
const phrase = ref<PhraseDetails | null>(null)
const explained = ref(false)
const error = ref('')

onMounted(async () => {
  try {
    phrase.value = await api<PhraseDetails>(`/phrases/${encodeURIComponent(props.phraseId)}`)
  } catch (cause) {
    error.value = String(cause)
  }
})
</script>

<template>
  <section class="card phrase-details">
    <button class="quiet back" @click="emit('back')">← Back</button>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <template v-if="phrase">
      <h1 lang="en">{{ phrase.text }}</h1>
      <div class="row-actions">
        <button class="primary" :disabled="disabled" @click="emit('practice', phrase.id)">
          Practice
        </button>
      </div>
      <div class="audio-controls" role="group" aria-label="Audio controls">
        <button :disabled="disabled" @click="play(phrase.reference_url)">
          <AudioIcon name="reference" /> Ref
        </button>
        <SavedHistory :phrase-id="phrase.id" :disabled="disabled" :play="play" />
      </div>
      <PhraseExplanation
        :explanation="phrase.explanation"
        :usage-contexts="phrase.usage_contexts"
        :accepted-variants="phrase.accepted_variants"
        :detailed-explanation="phrase.detailed_explanation"
        :open="explained"
        @toggle="explained = !explained"
      />
      <p>
        Lesson: {{ phrase.lesson_title }} · Source:
        <a v-if="phrase.source_link" :href="phrase.source_link" rel="noopener noreferrer">{{
          phrase.source_label
        }}</a>
        <template v-else>{{ phrase.source_label }}</template>
      </p>
      <p>{{ phrase.due_date ? `Next due: ${phrase.due_date}` : 'New: not rated yet' }}</p>
      <button
        class="danger"
        :disabled="disabled"
        @click="emit('remove', { id: phrase.id, text: phrase.text })"
      >
        Delete
      </button>
    </template>
  </section>
</template>
