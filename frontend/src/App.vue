<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { api } from './api'
import { useCapture } from './composables/useCapture'
import { setAudioSession } from './audioSession'
import { takeTransfers } from './takeTransfers'
import type { StoredTake } from './practiceTypes'
import SavedHistory from './components/SavedHistory.vue'
import AddMaterial from './components/AddMaterial.vue'

type Recording = { id: string; captured_at: string; duration_ms: number | null; url: string }
type Card = {
  id: string
  source_label: string
  answer: string
  explanation: string
  cues: Array<{ situation: string; partner_line?: string }>
  accepted_variants: string[]
  reference_url: string
  attempt_id: string
  recordings: Recording[]
}
type Study = {
  done: boolean
  total: number
  studied: number
  message?: string
  card?: Card
}
type HistoryItem = Pick<Card, 'id' | 'answer' | 'source_label' | 'reference_url' | 'recordings'>

const study = ref<Study | null>(null)
const loadingStudy = ref(false)
const connectionError = ref('')
const history = ref<HistoryItem[]>([])
let playing: HTMLAudioElement | null = null

async function loadStudy() {
  loadingStudy.value = true
  connectionError.value = ''
  try {
    ;[study.value, history.value] = await Promise.all([
      api<Study>('/study/current', { method: 'POST' }),
      api<HistoryItem[]>('/study/history'),
    ])
  } catch (cause) {
    connectionError.value = String(cause)
  } finally {
    loadingStudy.value = false
  }
}

function stopPlayback() {
  if (playing) {
    const current = playing
    playing = null
    current.pause()
    current.src = ''
  }
  setAudioSession('auto')
}

async function play(url: string) {
  stopPlayback()
  setAudioSession('playback')
  const audio = new Audio(url)
  playing = audio
  audio.onended = audio.onerror = () => {
    if (playing === audio) playing = null
    setAudioSession('auto')
  }
  await audio.play()
}

async function compare(recording: Recording, referenceUrl = study.value?.card?.reference_url) {
  stopPlayback()
  const learner = new Audio(recording.url)
  playing = learner
  learner.onended = () => {
    if (playing !== learner || !referenceUrl) return
    void play(referenceUrl)
  }
  learner.onpause = () => {
    if (playing !== learner) return
    learner.src = ''
    playing = null
    setAudioSession('auto')
  }
  learner.onerror = stopPlayback
  await learner.play()
}

const capture = useCapture({
  context: async () => {
    if (!study.value?.card) throw new Error('No study card is open')
    return { card_id: study.value.card.id, attempt_id: study.value.card.attempt_id }
  },
  retain: takeTransfers.retain,
  finished: async (_take: StoredTake) => {
    await takeTransfers.flush()
    await loadStudy()
  },
  failure: async () => {},
})

async function next() {
  if (!study.value?.card) return
  await capture.release()
  await takeTransfers.flush()
  await api(`/attempts/${study.value.card.attempt_id}/complete`, {
    method: 'POST',
    body: JSON.stringify({ operation_id: crypto.randomUUID() }),
  })
  stopPlayback()
  await loadStudy()
}

async function retryTakes() {
  await takeTransfers.flush()
  await loadStudy()
}

const progress = computed(() =>
  study.value ? `${study.value.studied} of ${study.value.total} studied` : '',
)

onMounted(async () => {
  await takeTransfers.recover()
  await loadStudy()
})
onUnmounted(() => {
  stopPlayback()
})
</script>

<template>
  <div class="shell">
    <header><strong>Phrase Gym</strong><span>English phrase study</span></header>
    <main>
      <AddMaterial @added="loadStudy" />

      <section v-if="connectionError" class="panel error" role="alert">
        {{ connectionError }} <button @click="loadStudy">Retry connection</button>
      </section>
      <section v-else-if="loadingStudy" class="panel" role="status">Loading…</section>
      <section v-else-if="study?.done" class="panel complete">
        <p class="eyebrow">{{ progress }}</p>
        <h1>Today’s study is complete</h1>
        <p>{{ study.message }}</p>
      </section>
      <section v-else-if="study?.card" class="panel study">
        <p class="eyebrow">First study · {{ progress }}</p>
        <h1>{{ study.card.answer }}</h1>
        <p class="meaning">{{ study.card.explanation }}</p>
        <p v-if="study.card.accepted_variants.length" class="variants">
          Also accepted: {{ study.card.accepted_variants.join('; ') }}
        </p>
        <p class="source">From {{ study.card.source_label }}</p>
        <div class="actions">
          <button @click="play(study.card.reference_url)">Play reference</button>
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
        <div v-if="study.card.recordings.length" class="takes">
          <h2>Saved takes</h2>
          <div v-for="(recording, index) in study.card.recordings" :key="recording.id" class="take">
            <span>Take {{ index + 1 }}</span>
            <button @click="play(recording.url)">Replay</button>
            <button @click="compare(recording)">Compare with reference</button>
          </div>
        </div>
        <button class="next" @click="next">Next</button>
        <p class="note">This first meeting is ungraded. Recording is optional.</p>
      </section>
      <section v-else class="panel empty">
        <h1>Add your first phrase batch</h1>
        <p>Import 5–10 prepared phrases above to begin.</p>
      </section>
      <SavedHistory :items="history" :play="play" :compare="compare" />
    </main>
  </div>
</template>
