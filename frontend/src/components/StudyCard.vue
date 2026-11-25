<script setup lang="ts">
import PhraseExplanation from './PhraseExplanation.vue'
import FrontControls from './FrontControls.vue'
import BackControls from './BackControls.vue'
import CardMenu from './CardMenu.vue'
import RecordingHistoryScreen from './RecordingHistoryScreen.vue'
import RatingButtons from './RatingButtons.vue'
import { computed, nextTick, ref, watch } from 'vue'
import type { DeletionTarget } from '../libraryTypes'
import { progressLine } from '../practiceReducer'
import type { Card, Outcome, Progress, Rating, Recording, ReviewedCard } from '../studyTypes'
import MicrophoneButton from './MicrophoneButton.vue'

const props = defineProps<{
  card: Card
  reviewedCards: ReviewedCard[]
  reviewIndex: number | null
  progress: Progress | null
  detailsOpen: boolean
  capturing: boolean
  requesting: boolean
  processing: boolean
  playingUrl: string | null
  busy: boolean
  hasTake: boolean
  failedTakes: number
  retryingSave: boolean
  captureError: string
  practiceError: string
  playRecording: (take: Recording) => Promise<void>
  stopPlayback: () => void
  canRetry: boolean
}>()
const emit = defineEmits<{
  control: []
  show: []
  details: []
  newCue: []
  hint: []
  complete: [outcome: Outcome]
  gradeSkipped: [completionId: string, rating: Rating]
  remove: [target: DeletionTarget]
  playPhrase: []
  playMine: []
  compare: []
  retrySave: []
  retryCompletion: []
  browse: [index: number | null]
}>()
const audioBusy = computed(() => props.capturing || props.requesting || props.processing)
const historyOpen = ref(false)
const menu = ref<InstanceType<typeof CardMenu> | null>(null)

function openHistory() {
  props.stopPlayback()
  historyOpen.value = true
}

async function closeHistory() {
  historyOpen.value = false
  await nextTick()
  menu.value?.focusToggle()
}
const previousDetailsOpen = ref(false)
const reviewed = computed(() =>
  props.reviewIndex === null ? null : props.reviewedCards[props.reviewIndex],
)
const savedGrade = computed(() =>
  reviewed.value?.outcome === 'skipped' ? undefined : reviewed.value?.outcome,
)
const activeCard = computed(() => reviewed.value?.card ?? props.card)
const activeProgress = computed(() => (reviewed.value ? reviewed.value.progress : props.progress))
const front = computed(() => !activeCard.value.revealed)
const showDock = computed(() => !front.value || props.reviewedCards.length > 0)
const previousIndex = computed(() => (props.reviewIndex ?? props.reviewedCards.length) - 1)
const phraseHeading = ref<HTMLElement | null>(null)

async function browse(index: number | null) {
  historyOpen.value = false
  previousDetailsOpen.value = false
  emit('browse', index)
  await nextTick()
  phraseHeading.value?.focus()
}

function forward() {
  const current = props.reviewIndex
  if (current === null) return emit('complete', 'skipped')
  const next = current + 1
  void browse(next < props.reviewedCards.length ? next : null)
}

function grade(rating: Rating) {
  if (!reviewed.value) emit('complete', rating)
  else if (reviewed.value.outcome === 'skipped')
    emit('gradeSkipped', reviewed.value.completionId, rating)
}

watch(
  () => props.card.view_token,
  () => {
    historyOpen.value = false
    previousDetailsOpen.value = false
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
  <RecordingHistoryScreen
    v-if="historyOpen"
    :phrase-id="activeCard.phrase_id"
    :playing-url="playingUrl"
    :play-recording="playRecording"
    :stop-playback="stopPlayback"
    @back="closeHistory"
  />
  <section v-show="!historyOpen" class="practice-panel" :class="{ 'has-ratings': showDock }">
    <div class="session-toolbar">
      <span data-testid="mode-line">{{ progressLine(activeCard, activeProgress) }}</span>
      <CardMenu
        ref="menu"
        :key="activeCard.view_token"
        :show-history="!front"
        :disabled="busy || audioBusy"
        @history="openHistory"
        @remove="
          emit('remove', { id: activeCard.phrase_id, text: activeCard.answer?.text ?? null })
        "
      />
    </div>
    <div class="practice-progress" aria-hidden="true">
      <span :style="{ width: `${Math.min(progressValue * 100, 100)}%` }" />
    </div>
    <article class="card practice-card study">
      <FrontControls
        v-if="front"
        :disabled="busy || audioBusy"
        :other-cue="card.other_cue_available"
        :hint="card.hint_available"
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
        :playing="!!playingUrl"
        :disabled="
          (processing && !playingUrl) || (busy && !capturing && !requesting && !playingUrl)
        "
        @control="emit('control')"
      />
      <div class="card-content" :key="activeCard.view_token">
        <p v-if="captureError" role="alert" class="error">{{ captureError }}</p>
        <p v-if="failedTakes" role="alert" class="error">
          {{ failedTakes }} {{ failedTakes === 1 ? 'recording' : 'recordings' }} could not be saved.
          Audio remains on this device.
          <button v-if="!retryingSave" @click="emit('retrySave')">Retry save</button>
        </p>
        <p v-if="retryingSave" role="status" class="muted">Saving recording…</p>
        <p v-if="practiceError" role="alert" class="error">
          {{ practiceError }}
          <button v-if="canRetry && !reviewed" :disabled="busy" @click="emit('retryCompletion')">
            Retry
          </button>
        </p>
        <template v-if="front">
          <p v-if="card.instruction" class="muted practice-instruction">{{ card.instruction }}</p>
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
            :disabled="!!savedGrade || busy || requesting || processing"
            :return-days="savedGrade ? null : activeCard.return_days"
            :selected="savedGrade"
            @complete="grade"
          />
          <button
            v-if="!front"
            class="rating-navigation"
            :aria-label="reviewed ? 'Next card' : 'Skip'"
            :title="reviewed ? 'Next card' : 'Skip'"
            :disabled="busy || requesting || processing || (!!reviewed && capturing)"
            @click="forward"
          >
            <span class="rating-chevron" aria-hidden="true" />
          </button>
        </div>
      </div>
    </article>
  </section>
</template>
