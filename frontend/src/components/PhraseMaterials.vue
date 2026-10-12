<script setup lang="ts">
import type { RouteLocationRaw } from 'vue-router'
import type { components } from '../api.generated'
import type { Language, Phrase } from '../api'
import type { Messages } from '../i18n'
defineProps<{
  t: Messages
  phrase: Phrase
  language: Language
  nativeLanguage: Language | null | undefined
  explanation: components['schemas']['Explanation'] | null | undefined
  playing: boolean
  audioError: string
  pendingExposure: unknown
  exposureSaving: boolean
  link: (kind: 'phrases' | 'lessons', id: string) => RouteLocationRaw
}>()
const emit = defineEmits<{ reference: []; retryExposure: [] }>()
</script>
<template>
  <article class="card phrase-card">
    <p class="eyebrow">{{ t.materials }}</p>
    <h1 :lang="language">{{ phrase.content.text }}</h1>
    <p :lang="language" class="purpose">{{ phrase.content.purpose }}</p>
    <button
      v-if="phrase.audio_status === 'ready'"
      class="reference"
      @click="emit('reference')"
      :aria-label="playing ? t.stop : t.reference"
    >
      {{ playing ? '■' : '▶' }} {{ playing ? t.stop : t.reference }}
    </button>
    <p v-else class="muted">{{ t.missing }}</p>
    <p v-if="audioError" role="alert" class="error">{{ audioError }}</p>
    <div class="explanation" v-if="explanation" :lang="nativeLanguage ?? undefined">
      <h2>{{ t.meaning }}</h2>
      <p>{{ explanation.meaning }}</p>
      <h2>{{ t.usage }}</h2>
      <p>{{ explanation.usage }}</p>
      <ul v-if="explanation.breakdown?.length">
        <li v-for="chunk in explanation.breakdown" :key="chunk">{{ chunk }}</li>
      </ul>
    </div>
    <p v-else>{{ t.noExplanation }}</p>
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
