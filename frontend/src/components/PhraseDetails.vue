<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api } from '../api'
import type { DeletionTarget, PhraseDetails } from '../libraryTypes'
import type { Recording } from '../studyTypes'
import SavedHistory from './SavedHistory.vue'

const props = defineProps<{
  phraseId: string
  disabled: boolean
  play: (url: string) => Promise<void>
  compare: (recording: Recording, referenceUrl: string) => Promise<void>
}>()
const emit = defineEmits<{ back: []; practise: [phraseId: string]; remove: [DeletionTarget] }>()
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
      <p class="purpose">Used for: {{ phrase.usage_contexts.join(' · ') }}</p>
      <p>{{ phrase.explanation }}</p>
      <p v-if="phrase.accepted_variants.length" class="muted">
        Also accepted: {{ phrase.accepted_variants.join('; ') }}
      </p>
      <div class="row-actions">
        <button :aria-expanded="explained" @click="explained = !explained">Details</button>
        <button :disabled="disabled" @click="play(phrase.reference_url)">▶ Phrase</button>
        <button :disabled="disabled" @click="emit('practise', phrase.id)">Practise freely</button>
      </div>
      <div v-if="explained" class="explanation">
        <h2>Details</h2>
        <p>{{ phrase.detailed_explanation ?? 'No detailed explanation was prepared.' }}</p>
      </div>
      <p>
        Lesson: {{ phrase.lesson_title }} · Source:
        <a v-if="phrase.source_link" :href="phrase.source_link" rel="noopener noreferrer">{{
          phrase.source_label
        }}</a>
        <template v-else>{{ phrase.source_label }}</template>
      </p>
      <p>{{ phrase.due_date ? `Next due: ${phrase.due_date}` : 'New: not studied yet' }}</p>
      <SavedHistory
        :phrase-id="phrase.id"
        :reference-url="phrase.reference_url"
        :disabled="disabled"
        :play="play"
        :compare="compare"
      />
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
