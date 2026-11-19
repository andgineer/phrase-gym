<script setup lang="ts">
import PhraseExplanation from './PhraseExplanation.vue'
import FrontControls from './FrontControls.vue'
import BackControls from './BackControls.vue'
import CardMenu from './CardMenu.vue'
import PracticeDialog from './PracticeDialog.vue'
import SavedHistoryPanel from './SavedHistoryPanel.vue'
import RatingButtons from './RatingButtons.vue'
import { computed, ref, watch } from 'vue'
import { progressLine } from '../practiceReducer'
import type { Card, Outcome, Progress, ReviewedCard } from '../studyTypes'
import MicrophoneButton from './MicrophoneButton.vue'

const props = defineProps<{
  card: Card
  previousCard: ReviewedCard | null
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
  play: (url: string) => Promise<void>
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
}>()
const audioBusy = computed(() => props.capturing || props.requesting || props.processing)
const front = computed(() => !props.card.revealed)
const panel = ref<'history' | 'previous' | null>(null)
const previousDetailsOpen = ref(false)
function reviewPrevious(event: MouseEvent) {
  if (event.currentTarget instanceof HTMLButtonElement) event.currentTarget.focus()
  previousDetailsOpen.value = false
  panel.value = 'previous'
}
watch(
  () => props.card.view_token,
  () => {
    panel.value = null
  },
)
const progressValue = computed(() => {
  const p = props.progress
  if (!p || props.card.mode === 'practice') return 0
  return props.card.kind === 'recall'
    ? p.due_completed / (p.due_allowance || 1)
    : p.new_completed / (p.new_allowance || 1)
})
</script>

<template>
  <section class="practice-panel" :class="{ 'has-ratings': !front }">
    <div class="session-toolbar">
      <span data-testid="mode-line">{{ progressLine(card, progress) }}</span>
      <CardMenu
        v-if="!front"
        :disabled="busy || audioBusy"
        @history="panel = 'history'"
        @remove="emit('remove')"
      />
      <span v-else class="menu-space" />
    </div>
    <div class="practice-progress" aria-hidden="true">
      <span :style="{ width: `${Math.min(progressValue * 100, 100)}%` }" />
    </div>
    <article class="card practice-card study">
      <FrontControls
        v-if="front"
        :disabled="busy || audioBusy"
        :other-cue="card.other_cue_available"
        @hint="emit('hint')"
        @new-cue="emit('newCue')"
        @reveal="emit('show')"
      />
      <BackControls
        v-else
        :disabled="audioBusy"
        :has-take="hasTake"
        @reference="emit('playPhrase')"
        @mine="emit('playMine')"
        @compare="emit('compare')"
      />
      <MicrophoneButton
        :capturing="capturing"
        :requesting="requesting"
        :playing="playing"
        :disabled="(processing && !playing) || (busy && !capturing && !requesting && !playing)"
        @control="emit('control')"
      />
      <div class="card-content">
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
        <template v-if="front">
          <p class="practice-prompt">{{ card.cue?.situation }}</p>
          <p v-if="card.cue?.partner_line" class="partner-line">“{{ card.cue.partner_line }}”</p>
          <p v-if="card.hint" class="hint" role="status">Hint: {{ card.hint }}</p>
        </template>
        <template v-else-if="card.answer">
          <h2 tabindex="-1" lang="en" data-testid="practice-phrase">{{ card.answer.text }}</h2>
          <PhraseExplanation
            :explanation="card.answer.explanation"
            :usage-contexts="card.answer.usage_contexts"
            :detailed-explanation="card.answer.detailed_explanation"
            :open="detailsOpen"
            @toggle="emit('details')"
          />
        </template>
      </div>
      <div v-if="!front" class="rating-dock">
        <div class="ratings">
          <button
            class="rating-navigation"
            aria-label="Previous card"
            title="Previous card"
            aria-haspopup="dialog"
            :disabled="!previousCard || busy || audioBusy"
            @click="reviewPrevious"
          >
            <span class="rating-chevron previous-chevron" aria-hidden="true" />
          </button>
          <RatingButtons
            :disabled="busy || requesting || processing"
            :return-days="card.return_days"
            @complete="emit('complete', $event)"
          />
        </div>
      </div>
    </article>
    <PracticeDialog
      v-if="!front && panel === 'history'"
      title="Recording history"
      @close="panel = null"
    >
      <SavedHistoryPanel :phrase-id="card.phrase_id" :disabled="audioBusy" :play="play" />
    </PracticeDialog>
    <PracticeDialog
      v-else-if="!front && panel === 'previous' && previousCard"
      title="Previous card"
      @close="panel = null"
    >
      <h3 class="review-phrase" lang="en">{{ previousCard.answer.text }}</h3>
      <PhraseExplanation
        :explanation="previousCard.answer.explanation"
        :usage-contexts="previousCard.answer.usage_contexts"
        :detailed-explanation="previousCard.answer.detailed_explanation"
        :open="previousDetailsOpen"
        @toggle="previousDetailsOpen = !previousDetailsOpen"
      />
      <div class="ratings review-ratings">
        <RatingButtons disabled :selected="previousCard.outcome" />
      </div>
      <p v-if="previousCard.outcome === 'skipped'" class="muted">Skipped without a grade</p>
    </PracticeDialog>
  </section>
</template>
