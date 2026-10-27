<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useCapture } from './composables/useCapture'
import { takeTransfers } from './takeTransfers'
import { resetDiscardedLearnerDataOnce } from './discardedLearnerData'
import type { StoredTake } from './practiceTypes'
import SavedHistory from './components/SavedHistory.vue'
import AddMaterial from './components/AddMaterial.vue'
import StudyCard from './components/StudyCard.vue'
import { usePlayback } from './composables/usePlayback'
import { useStudy } from './composables/useStudy'

const study = useStudy()
const playback = usePlayback()
const initialized = ref(false)
let mediaScope = 0
let compareOnStop: number | null = null
const capture = useCapture({
  context: async () => {
    if (!study.current.value?.card) throw new Error('No study card is open')
    return {
      card_id: study.current.value.card.id,
      attempt_id: study.current.value.card.attempt_id,
    }
  },
  retain: takeTransfers.retain,
  finished: async (take: StoredTake) => {
    const shouldCompare =
      take.complete &&
      compareOnStop === mediaScope &&
      study.current.value?.card?.id === take.card_id &&
      study.current.value.card.attempt_id === take.attempt_id &&
      !document.hidden
    compareOnStop = null
    if (shouldCompare) {
      const reference = study.current.value?.card?.reference_url
      if (reference) void playback.compareTake(take, reference)
    }
    await takeTransfers.flush()
    await study.load()
  },
  failure: async () => {},
})

async function next() {
  if (!study.current.value?.card) return
  mediaScope++
  compareOnStop = null
  playback.stop()
  await study.complete(study.current.value.card.attempt_id, async () => {
    await capture.release()
    await takeTransfers.flush()
    playback.stop()
  })
}

async function retryTakes() {
  await takeTransfers.flush()
  await study.load()
}

async function controlAudio() {
  if (capture.requesting.value) return capture.release()
  if (capture.capturing.value) {
    compareOnStop = mediaScope
    return capture.stop()
  }
  if (playback.active.value) return playback.stop()
  mediaScope++
  compareOnStop = null
  playback.stop()
  return capture.start()
}

async function beforeOpenIntake() {
  mediaScope++
  compareOnStop = null
  playback.stop()
  if (capture.capturing.value) await capture.stop()
  await capture.release()
  await takeTransfers.flush()
  await study.load()
}

function playReference(url: string) {
  return playback.play(url)
}

const progress = computed(() =>
  study.current.value
    ? `${study.current.value.studied + (study.current.value.done ? 0 : 1)} / ${study.current.value.total}`
    : '',
)

async function initialize() {
  initialized.value = false
  try {
    await resetDiscardedLearnerDataOnce()
    await takeTransfers.recover()
    initialized.value = true
    await study.load()
  } catch (cause) {
    study.error.value = String(cause)
  }
}

onMounted(initialize)
</script>

<template>
  <div class="shell">
    <header>
      <a class="brand" href="/">Phrase Gym<span>◌</span></a>
      <AddMaterial v-if="initialized" :before-open="beforeOpenIntake" @added="study.load" />
    </header>
    <main>
      <p v-if="study.error.value" class="error" role="alert">
        {{ study.error.value }} <button @click="initialize">Retry connection</button>
      </p>
      <p v-else-if="!initialized || (study.loading.value && !study.current.value)" role="status">
        Loading…
      </p>
      <section v-else-if="study.current.value?.done" class="card complete">
        <p class="eyebrow">English · {{ progress }}</p>
        <h1>Today’s study is complete</h1>
        <p>{{ study.current.value.message }}</p>
      </section>
      <template v-else-if="study.current.value?.card">
        <p class="eyebrow">English</p>
        <StudyCard
          :card="study.current.value.card"
          :progress="progress"
          :capturing="capture.capturing.value"
          :requesting="capture.requesting.value"
          :processing="capture.processing.value"
          :playing="playback.active.value"
          :completing="study.completing.value"
          :pending-takes="takeTransfers.pendingTakes.value"
          :capture-error="capture.error.value"
          :take-error="takeTransfers.error.value"
          :play="playReference"
          :compare="playback.compare"
          @control="controlAudio"
          @retry="retryTakes"
          @next="next"
        />
      </template>
      <section v-else class="home">
        <p class="eyebrow">English</p>
        <h1>Phrase Gym</h1>
        <div class="card session-entry">
          <p>Add your first phrase batch</p>
          <p class="muted">Import 5–10 prepared phrases to begin.</p>
        </div>
      </section>
      <p v-if="playback.error.value" class="error" role="alert">{{ playback.error.value }}</p>
      <SavedHistory
        :items="study.history.value"
        :play="playback.play"
        :compare="playback.compare"
        :disabled="capture.capturing.value || capture.requesting.value || capture.processing.value"
      />
    </main>
  </div>
</template>
