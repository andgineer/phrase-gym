<script setup lang="ts">
import { computed } from 'vue'
import { progressLine } from '../practiceReducer'
import type { Card, Outcome, Progress } from '../studyTypes'
import MicrophoneButton from './MicrophoneButton.vue'

const props = defineProps<{
  card: Card
  progress: Progress | null
  detailsOpen: boolean
  capturing: boolean
  requesting: boolean
  processing: boolean
  playing: boolean
  busy: boolean
  hasTake: boolean
  pendingTakes: number
  captureError: string
  takeError: string
  practiceError: string
  canRetry: boolean
}>()
const emit = defineEmits<{
  control: []
  show: []
  details: []
  newCue: []
  hint: []
  complete: [outcome: Outcome]
  remove: []
  playPhrase: []
  playMine: []
  compare: []
  retrySave: []
  retryCompletion: []
  stop: []
}>()
const ratings: Array<[Outcome, string, string]> = [
  ['not_yet', 'Not yet', 'not-yet'],
  ['effort', 'Effort', 'effort'],
  ['got_it', 'Got it', 'recalled'],
]
const audioBusy = computed(() => props.capturing || props.requesting || props.processing)
const front = computed(() => props.card.kind === 'recall' && !props.card.revealed)
</script>

<template>
  <section class="practice-panel">
    <div class="session-toolbar">
      <span data-testid="mode-line">{{ progressLine(card, progress) }}</span>
      <button v-if="card.mode === 'extra'" @click="emit('stop')">Stop</button>
    </div>
    <article class="card practice-card study">
      <template v-if="front">
        <p class="practice-prompt">{{ card.cue?.situation }}</p>
        <p v-if="card.cue?.partner_line" class="partner-line">“{{ card.cue.partner_line }}”</p>
        <div class="answer-controls">
          <button
            v-if="card.other_cue_available"
            :disabled="busy || audioBusy"
            @click="emit('newCue')"
          >
            New cue
          </button>
          <button :disabled="busy || audioBusy" @click="emit('hint')">Hint</button>
        </div>
        <p v-if="card.hint" class="hint" role="status">Hint: {{ card.hint }}</p>
      </template>
      <template v-else-if="card.answer">
        <h1 tabindex="-1" lang="en" data-testid="practice-phrase">{{ card.answer.text }}</h1>
        <div class="answer-help">
          <p>{{ card.answer.explanation }}</p>
        </div>
        <p v-if="card.answer.accepted_variants.length" class="muted">
          Also accepted: {{ card.answer.accepted_variants.join('; ') }}
        </p>
      </template>
      <div class="audio-controls">
        <MicrophoneButton
          :capturing="capturing"
          :requesting="requesting"
          :playing="playing"
          :disabled="(processing && !playing) || (busy && !capturing && !requesting && !playing)"
          @control="emit('control')"
        />
      </div>
      <div v-if="front" class="answer-controls">
        <button class="primary" :disabled="busy || audioBusy" @click="emit('show')">Show</button>
        <button :disabled="busy || audioBusy" @click="emit('details')">Details</button>
      </div>
      <template v-else-if="card.answer">
        <div class="answer-controls">
          <button :aria-expanded="detailsOpen" @click="emit('details')">Details</button>
          <button :disabled="audioBusy" @click="emit('playPhrase')">▶ Phrase</button>
          <template v-if="hasTake">
            <button :disabled="audioBusy" @click="emit('playMine')">▶ Mine</button>
            <button :disabled="audioBusy" @click="emit('compare')">Compare</button>
          </template>
        </div>
        <div v-if="detailsOpen" class="explanation">
          <h2>Details</h2>
          <p>{{ card.answer.detailed_explanation ?? 'No detailed explanation was prepared.' }}</p>
          <p class="purpose">Used for: {{ card.answer.usage_contexts.join(' · ') }}</p>
        </div>
        <template v-if="card.kind === 'recall'">
          <p>How was your answer before you saw the phrase?</p>
          <div class="ratings">
            <button
              v-for="[outcome, label, style] in ratings"
              :key="outcome"
              :class="style"
              :disabled="busy || requesting || processing"
              @click="emit('complete', outcome)"
            >
              {{ label }}
            </button>
            <button
              :disabled="busy || requesting || processing"
              @click="emit('complete', 'skipped')"
            >
              Skip
            </button>
          </div>
        </template>
        <button class="danger delete-card" :disabled="busy || audioBusy" @click="emit('remove')">
          Delete
        </button>
        <template v-if="card.kind === 'first_study'">
          <p class="muted">First meeting · no rating</p>
          <button
            class="primary study-next"
            :disabled="busy || requesting || processing"
            @click="emit('complete', 'studied')"
          >
            Next
          </button>
        </template>
      </template>
      <p v-if="capturing" role="status">Tap the microphone again to finish your take.</p>
      <p v-if="requesting" role="status">Waiting for microphone access…</p>
      <p v-if="processing" role="status">Finishing and saving the complete take…</p>
      <p v-if="captureError" role="alert" class="error">{{ captureError }}</p>
      <p v-if="pendingTakes" role="status">
        {{ pendingTakes }} take(s) waiting to save.
        <button @click="emit('retrySave')">Retry save</button>
      </p>
      <p v-if="takeError" role="alert" class="error">{{ takeError }}</p>
      <p v-if="practiceError" role="alert" class="error">
        {{ practiceError }}
        <button v-if="canRetry" :disabled="busy" @click="emit('retryCompletion')">Retry</button>
      </p>
    </article>
  </section>
</template>
