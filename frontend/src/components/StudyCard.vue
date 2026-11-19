<script setup lang="ts">
import PhraseExplanation from './PhraseExplanation.vue'
import FrontControls from './FrontControls.vue'
import BackControls from './BackControls.vue'
import CardMenu from './CardMenu.vue'
import PracticeDialog from './PracticeDialog.vue'
import SavedHistoryPanel from './SavedHistoryPanel.vue'
import RatingButtons from './RatingButtons.vue'
import { computed, nextTick, ref, watch } from 'vue'
import { progressLine } from '../practiceReducer'
import type { Card, Outcome, Progress, Recording, ReviewedCard } from '../studyTypes'
import MicrophoneButton from './MicrophoneButton.vue'

const props = defineProps<{
  card: Card
  reviewedCards: ReviewedCard[]
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
  compareSaved: (take: Recording, referenceUrl?: string) => Promise<void>
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
  stopPlayback: []
}>()
const audioBusy = computed(() => props.capturing || props.requesting || props.processing)
const historyOpen = ref(false)
const reviewIndex = ref<number | null>(null)
const previousDetailsOpen = ref(false)
const reviewed = computed(() =>
  reviewIndex.value === null ? null : props.reviewedCards[reviewIndex.value],
)
const activeCard = computed(() => reviewed.value?.card ?? props.card)
const activeProgress = computed(() => (reviewed.value ? reviewed.value.progress : props.progress))
const front = computed(() => !activeCard.value.revealed)
const showDock = computed(() => !front.value || props.reviewedCards.length > 0)
const previousIndex = computed(() => (reviewIndex.value ?? props.reviewedCards.length) - 1)
const savedTake = computed(() => reviewed.value?.card.recordings.at(-1))
const phraseHeading = ref<HTMLElement | null>(null)

async function browse(index: number | null) {
  emit('stopPlayback')
  historyOpen.value = false
  previousDetailsOpen.value = false
  reviewIndex.value = index
  await nextTick()
  phraseHeading.value?.focus()
}

function forward() {
  const current = reviewIndex.value
  if (current === null) return emit('complete', 'skipped')
  const next = current + 1
  void browse(next < props.reviewedCards.length ? next : null)
}

function playReference() {
  if (!reviewed.value) return emit('playPhrase')
  const url = activeCard.value.answer?.reference_url
  if (url) void props.play(url)
}

function playMine() {
  if (!reviewed.value) return emit('playMine')
  if (savedTake.value) void props.play(savedTake.value.url)
}

function compare() {
  if (!reviewed.value) return emit('compare')
  if (savedTake.value)
    void props.compareSaved(savedTake.value, activeCard.value.answer?.reference_url)
}

watch(
  () => props.card.view_token,
  () => {
    historyOpen.value = false
    reviewIndex.value = null
  },
)
const progressValue = computed(() => {
  const p = activeProgress.value
  if (!p || activeCard.value.mode === 'practice') return 0
  return activeCard.value.kind === 'recall'
    ? p.due_completed / (p.due_allowance || 1)
    : p.new_completed / (p.new_allowance || 1)
})
</script>

<template>
  <section class="practice-panel" :class="{ 'has-ratings': showDock }">
    <div class="session-toolbar">
      <span data-testid="mode-line">{{ progressLine(activeCard, activeProgress) }}</span>
      <CardMenu
        v-if="!front"
        :key="activeCard.view_token"
        :read-only="!!reviewed"
        :disabled="busy || audioBusy"
        @history="historyOpen = true"
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
        :has-take="reviewed ? !!savedTake : hasTake"
        @reference="playReference"
        @mine="playMine"
        @compare="compare"
      />
      <MicrophoneButton
        :capturing="capturing"
        :requesting="requesting"
        :playing="playing"
        :disabled="
          (reviewed && !playing) ||
          (processing && !playing) ||
          (busy && !capturing && !requesting && !playing)
        "
        @control="reviewed ? emit('stopPlayback') : emit('control')"
      />
      <div class="card-content" :key="activeCard.view_token">
        <template v-if="!reviewed">
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
        </template>
        <template v-if="front">
          <p ref="phraseHeading" tabindex="-1" class="practice-prompt">{{ card.cue?.situation }}</p>
          <p v-if="card.cue?.partner_line" class="partner-line">“{{ card.cue.partner_line }}”</p>
          <p v-if="card.hint" class="hint" role="status">Hint: {{ card.hint }}</p>
        </template>
        <template v-else-if="activeCard.answer">
          <h2 ref="phraseHeading" tabindex="-1" lang="en" data-testid="practice-phrase">
            {{ activeCard.answer.text }}
          </h2>
          <p v-if="reviewed?.outcome === 'skipped'" class="muted">Skipped without a grade</p>
          <PhraseExplanation
            :explanation="activeCard.answer.explanation"
            :usage-contexts="activeCard.answer.usage_contexts"
            :detailed-explanation="activeCard.answer.detailed_explanation"
            :open="reviewed ? previousDetailsOpen : detailsOpen"
            @toggle="reviewed ? (previousDetailsOpen = !previousDetailsOpen) : emit('details')"
          />
        </template>
      </div>
      <div v-if="showDock" class="rating-dock">
        <div class="ratings">
          <button
            class="rating-navigation"
            aria-label="Previous card"
            title="Previous card"
            :disabled="previousIndex < 0 || busy || audioBusy"
            @click="browse(previousIndex)"
          >
            <span class="rating-chevron previous-chevron" aria-hidden="true" />
          </button>
          <RatingButtons
            v-if="!front"
            :disabled="!!reviewed || busy || requesting || processing"
            :return-days="reviewed ? null : card.return_days"
            :selected="reviewed?.outcome"
            @complete="emit('complete', $event)"
          />
          <button
            v-if="!front"
            class="rating-navigation"
            :aria-label="reviewed ? 'Next card' : 'Skip'"
            :title="reviewed ? 'Next card' : 'Skip'"
            :disabled="!reviewed && (busy || requesting || processing)"
            @click="forward"
          >
            <span class="rating-chevron" aria-hidden="true" />
          </button>
        </div>
      </div>
    </article>
    <PracticeDialog
      v-if="!front && historyOpen"
      title="Recording history"
      @close="historyOpen = false"
    >
      <SavedHistoryPanel :phrase-id="activeCard.phrase_id" :disabled="audioBusy" :play="play" />
    </PracticeDialog>
  </section>
</template>
