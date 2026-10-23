<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { RouteLocationRaw } from 'vue-router'
import LookupRecording from './LookupRecording.vue'
import type { PracticeMessages } from '../practiceMessages'
import type { ExposureInput } from '../api'
import type { components } from '../api.generated'
import type { Language, Phrase } from '../api'
import type { Messages } from '../i18n'
import { preparedExplanation } from '../explanation'
import PhraseDetails from './PhraseDetails.vue'
const props = defineProps<{
  pt: PracticeMessages
  currentExposure: ExposureInput | null
  saveExposure: () => Promise<void>
  t: Messages
  phrase: Phrase
  language: Language
  nativeLanguage: Language | null | undefined
  explanation: components['schemas']['Explanation'] | null | undefined
  pendingExposure: unknown
  exposureSaving: boolean
  link: (kind: 'phrases' | 'lessons', id: string) => RouteLocationRaw
}>()
const prepared = computed(() => preparedExplanation(props.explanation))
const task = computed(() => props.phrase.english?.task)
const examples = computed(() =>
  task.value?.kind === 'word_sense'
    ? task.value.contexts.map((context) => ({
        text: context.answer,
        audio: context.reference_audio_id,
      }))
    : [],
)
const showDetails = ref(false)
watch(
  () => props.phrase.id,
  () => (showDetails.value = false),
)
const emit = defineEmits<{ retryExposure: [] }>()
</script>
<template>
  <article class="card phrase-card">
    <p class="eyebrow">{{ prepared ? t.materials : t.reference }}</p>
    <template v-if="task?.kind === 'word_sense'">
      <h1 :lang="language">{{ task.word }}</h1>
      <p class="word-sense" :lang="language">{{ task.sense }}</p>
    </template>
    <h1 v-else :lang="language">{{ phrase.content.text }}</h1>
    <p v-if="!phrase.english" :lang="language" class="purpose">{{ phrase.content.purpose }}</p>
    <template v-if="task?.kind !== 'word_sense'">
      <p
        v-for="situation in phrase.english?.situations"
        :key="situation.id"
        :lang="situation.language"
      >
        {{ situation.text }}
      </p>
    </template>
    <LookupRecording
      :phrase="phrase"
      :exposure="currentExposure"
      :save-exposure="saveExposure"
      :pt="pt"
      :examples="examples"
    />
    <p v-if="phrase.audio_status !== 'ready'" class="muted">{{ t.missing }}</p>
    <div
      class="explanation"
      v-if="prepared"
      :lang="phrase.english?.help?.language ?? nativeLanguage ?? undefined"
    >
      <template v-if="prepared.meaning.trim()">
        <h2>{{ t.meaning }}</h2>
        <p>{{ prepared.meaning }}</p>
      </template>
      <template v-if="prepared.usage?.trim()">
        <h2>{{ t.usage }}</h2>
        <p>{{ prepared.usage }}</p>
      </template>
      <ul v-if="prepared.breakdown?.length">
        <li v-for="chunk in prepared.breakdown" :key="chunk">{{ chunk }}</li>
      </ul>
    </div>
    <button
      v-if="phrase.english?.details"
      class="explain-link"
      :aria-expanded="showDetails"
      @click="showDetails = !showDetails"
    >
      📖 {{ pt.explain }}
    </button>
    <PhraseDetails v-if="showDetails" :phrase="phrase" />
    <p v-if="exposureSaving" role="status">{{ t.exposureSaving }}</p>
    <div v-else-if="pendingExposure" class="error" role="alert">
      {{ t.exposureFailed }} <button @click="emit('retryExposure')">{{ t.retry }}</button>
    </div>
    <nav class="lesson-links">
      <span>{{ t.lessons }}</span
      ><RouterLink
        v-for="lesson in phrase.lessons"
        :key="lesson.id"
        :to="link('lessons', lesson.id)"
        >{{ lesson.title }} →</RouterLink
      >
    </nav>
  </article>
</template>
