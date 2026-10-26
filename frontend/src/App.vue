<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useCapture } from './composables/useCapture'
import { takeTransfers } from './takeTransfers'
import type { StoredTake } from './practiceTypes'
import SavedHistory from './components/SavedHistory.vue'
import AddMaterial from './components/AddMaterial.vue'
import { usePlayback } from './composables/usePlayback'
import { useStudy } from './composables/useStudy'

const study = useStudy()
const playback = usePlayback()

const capture = useCapture({
  context: async () => {
    if (!study.current.value?.card) throw new Error('No study card is open')
    return {
      card_id: study.current.value.card.id,
      attempt_id: study.current.value.card.attempt_id,
    }
  },
  retain: takeTransfers.retain,
  finished: async (_take: StoredTake) => {
    await takeTransfers.flush()
    await study.load()
  },
  failure: async () => {},
})

async function next() {
  if (!study.current.value?.card) return
  await capture.release()
  await takeTransfers.flush()
  playback.stop()
  await study.complete(study.current.value.card.attempt_id)
}

async function retryTakes() {
  await takeTransfers.flush()
  await study.load()
}

const progress = computed(() =>
  study.current.value
    ? `${study.current.value.studied} of ${study.current.value.total} studied`
    : '',
)

onMounted(async () => {
  await takeTransfers.recover()
  await study.load()
})
</script>

<template>
  <div class="shell">
    <header><strong>Phrase Gym</strong><span>English phrase study</span></header>
    <main>
      <AddMaterial @added="study.load" />

      <section v-if="study.error.value" class="panel error" role="alert">
        {{ study.error.value }} <button @click="study.load">Retry connection</button>
      </section>
      <section v-else-if="study.loading.value" class="panel" role="status">Loading…</section>
      <section v-else-if="study.current.value?.done" class="panel complete">
        <p class="eyebrow">{{ progress }}</p>
        <h1>Today’s study is complete</h1>
        <p>{{ study.current.value.message }}</p>
      </section>
      <section v-else-if="study.current.value?.card" class="panel study">
        <p class="eyebrow">First study · {{ progress }}</p>
        <h1>{{ study.current.value.card.answer }}</h1>
        <p class="meaning">{{ study.current.value.card.explanation }}</p>
        <p v-if="study.current.value.card.accepted_variants.length" class="variants">
          Also accepted: {{ study.current.value.card.accepted_variants.join('; ') }}
        </p>
        <p class="source">From {{ study.current.value.card.source_label }}</p>
        <div class="actions">
          <button @click="playback.play(study.current.value.card.reference_url)">
            Play reference
          </button>
          <button
            v-if="!capture.capturing.value"
            :disabled="capture.requesting.value || capture.processing.value"
            @click="capture.start"
          >
            Record a take
          </button>
          <button v-else @click="capture.stop()">Stop recording</button>
        </div>
        <p v-if="capture.processing.value" role="status">Finishing and saving the complete take…</p>
        <p v-if="capture.error.value" role="alert" class="error">{{ capture.error.value }}</p>
        <p v-if="takeTransfers.pendingTakes.value" role="status">
          {{ takeTransfers.pendingTakes.value }} take(s) waiting to save.
          <button @click="retryTakes">Retry save</button>
        </p>
        <p v-if="takeTransfers.error.value" role="alert" class="error">
          {{ takeTransfers.error.value }}
        </p>
        <div v-if="study.current.value.card.recordings.length" class="takes">
          <h2>Saved takes</h2>
          <div
            v-for="(recording, index) in study.current.value.card.recordings"
            :key="recording.id"
            class="take"
          >
            <span>Take {{ index + 1 }}</span>
            <button @click="playback.play(recording.url)">Replay</button>
            <button @click="playback.compare(recording, study.current.value.card?.reference_url)">
              Compare with reference
            </button>
          </div>
        </div>
        <button class="next" @click="next">Next</button>
        <p class="note">This first meeting is ungraded. Recording is optional.</p>
      </section>
      <section v-else class="panel empty">
        <h1>Add your first phrase batch</h1>
        <p>Import 5–10 prepared phrases above to begin.</p>
      </section>
      <SavedHistory
        :items="study.history.value"
        :play="playback.play"
        :compare="playback.compare"
      />
    </main>
  </div>
</template>
