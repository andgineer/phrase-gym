<script setup lang="ts">
import PhraseExplanation from './PhraseExplanation.vue'
import AudioIcon from './AudioIcon.vue'
import { computed } from 'vue'
import { progressLine } from '../practiceReducer'
import { returnDescription, returnSpan } from '../returnSpan'
import type { Card, Outcome, Progress, Rating } from '../studyTypes'
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
const ratings: Array<[Rating, string, string]> = [
  ['not_yet', 'Again', 'not-yet'],
  ['effort', 'Hard', 'effort'],
  ['got_it', 'Good', 'recalled'],
  ['easy', 'Easy', 'recalled'],
]
const audioBusy = computed(() => props.capturing || props.requesting || props.processing)
const front = computed(() => !props.card.revealed)
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
      </template>
      <div class="audio-controls" role="group" aria-label="Audio controls">
        <button v-if="!front && card.answer" :disabled="audioBusy" @click="emit('playPhrase')">
          <AudioIcon name="reference" /> Ref
        </button>
        <MicrophoneButton
          :capturing="capturing"
          :requesting="requesting"
          :playing="playing"
          :disabled="(processing && !playing) || (busy && !capturing && !requesting && !playing)"
          @control="emit('control')"
        />
        <template v-if="!front && card.answer">
          <button v-if="hasTake" :disabled="audioBusy" @click="emit('playMine')">
            <AudioIcon name="mine" /> Mine
          </button>
          <button v-if="hasTake" :disabled="audioBusy" @click="emit('compare')">
            <AudioIcon name="compare" /> Compare
          </button>
          <slot name="history" />
        </template>
      </div>
      <div v-if="front" class="answer-controls">
        <button class="primary" :disabled="busy || audioBusy" @click="emit('show')">Show</button>
      </div>
      <template v-else-if="card.answer">
        <PhraseExplanation
          :explanation="card.answer.explanation"
          :usage-contexts="card.answer.usage_contexts"
          :detailed-explanation="card.answer.detailed_explanation"
          :open="detailsOpen"
          @toggle="emit('details')"
        />
        <p>How was your answer before you saw the phrase?</p>
        <div class="ratings">
          <button
            v-for="[outcome, label, style] in ratings"
            :key="outcome"
            :class="style"
            :title="card.return_days ? returnDescription(card.return_days[outcome]) : undefined"
            :disabled="busy || requesting || processing"
            @click="emit('complete', outcome)"
          >
            <span v-if="card.return_days" class="rating-return" aria-hidden="true">{{
              returnSpan(card.return_days[outcome])
            }}</span>
            {{ label }}
          </button>
          <button :disabled="busy || requesting || processing" @click="emit('complete', 'skipped')">
            Skip
          </button>
        </div>
        <button class="danger delete-card" :disabled="busy || audioBusy" @click="emit('remove')">
          Delete
        </button>
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
