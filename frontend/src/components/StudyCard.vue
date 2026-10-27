<script setup lang="ts">
import type { Card, Recording } from '../studyTypes'

defineProps<{
  card: Card
  progress: string
  capturing: boolean
  requesting: boolean
  processing: boolean
  playing: boolean
  completing: boolean
  pendingTakes: number
  captureError: string
  takeError: string
  play: (url: string) => Promise<void>
  compare: (recording: Recording, referenceUrl: string) => Promise<void>
}>()
const emit = defineEmits<{
  control: []
  retry: []
  next: []
}>()
</script>

<template>
  <section class="practice-panel">
    <div class="session-toolbar">
      <span>{{ progress }}</span>
    </div>
    <article class="card practice-card study">
      <p role="status">First study. Read and say the phrase.</p>
      <p v-if="card.cues[0]" class="practice-prompt">{{ card.cues[0].situation }}</p>
      <p v-if="card.cues[0]?.partner_line">{{ card.cues[0].partner_line }}</p>
      <h1 tabindex="-1" lang="en" data-testid="practice-phrase">{{ card.answer }}</h1>
      <p v-if="card.accepted_variants.length" class="muted">
        Also accepted: {{ card.accepted_variants.join('; ') }}
      </p>
      <div class="audio-controls">
        <button
          class="microphone"
          :class="{ capturing }"
          :disabled="processing || (completing && !capturing && !requesting && !playing)"
          :aria-label="
            requesting
              ? 'Cancel microphone request'
              : capturing
                ? 'Stop recording'
                : playing
                  ? 'Stop playback'
                  : 'Record a take'
          "
          @click="emit('control')"
        >
          <span v-if="capturing || requesting || playing" aria-hidden="true" class="stop-symbol" />
          <svg
            v-else
            aria-hidden="true"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <rect x="9" y="2" width="6" height="12" rx="3" />
            <path d="M5 10a7 7 0 0 0 14 0M12 17v5m-4 0h8" />
          </svg>
        </button>
        <button
          :disabled="!card.recordings.length || capturing || requesting || processing"
          @click="play(card.recordings.at(-1)!.url)"
        >
          ▶ My recording
        </button>
        <button :disabled="capturing || requesting || processing" @click="play(card.reference_url)">
          ▶ Reference
        </button>
      </div>
      <div class="answer-help">
        <p>{{ card.explanation }}</p>
      </div>
      <p v-if="capturing" role="status">Tap the microphone again to finish your take.</p>
      <p v-if="requesting" role="status">Waiting for microphone access…</p>
      <p v-if="processing" role="status">Finishing and saving the complete take…</p>
      <p v-if="captureError" role="alert" class="error">{{ captureError }}</p>
      <p v-if="pendingTakes" role="status">
        {{ pendingTakes }} take(s) waiting to save.
        <button @click="emit('retry')">Retry save</button>
      </p>
      <p v-if="takeError" role="alert" class="error">{{ takeError }}</p>
      <div v-if="card.recordings.length" class="saved-takes">
        <h2>Saved takes</h2>
        <div v-for="(recording, index) in card.recordings" :key="recording.id" class="take">
          <span>Take {{ index + 1 }}</span>
          <button @click="play(recording.url)">Replay</button>
          <button @click="compare(recording, card.reference_url)">Compare with reference</button>
        </div>
      </div>
      <button
        class="primary study-next"
        :disabled="completing || capturing || requesting || processing"
        @click="emit('next')"
      >
        Next
      </button>
      <p class="muted">This first meeting is ungraded. Recording is optional.</p>
    </article>
  </section>
</template>
