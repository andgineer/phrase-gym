<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { takeTransfers } from './takeTransfers'
import { resetDiscardedLearnerDataOnce } from './discardedLearnerData'
import type { DeletionTarget } from './libraryTypes'
import SettingsPanel from './components/SettingsPanel.vue'
import AddMaterial from './components/AddMaterial.vue'
import CleanupNotice from './components/CleanupNotice.vue'
import DeleteDialog from './components/DeleteDialog.vue'
import PracticeEntry from './components/PracticeEntry.vue'
import TakeRejections from './components/TakeRejections.vue'
import StudyCard from './components/StudyCard.vue'
import { usePracticeSession } from './composables/usePracticeSession'

const session = usePracticeSession()
const { practice, playback, capture } = session
const initialized = ref(false)
const startupError = ref('')
const screen = ref<'entry' | 'card' | 'settings'>('entry')
const lessonId = ref('')
const entryKey = ref(0)
const pendingDelete = ref<DeletionTarget | null>(null)
const card = computed(() => practice.view.value.card)

// An exhausted traversal returns to entry with its completion message.
watch(card, (current) => {
  if (!current && screen.value === 'card') screen.value = 'entry'
})

async function daily() {
  await practice.openDaily(lessonId.value || null)
  if (card.value) screen.value = 'card'
}

async function navigate(next: 'entry' | 'settings') {
  await session.leave()
  void takeTransfers.flush()
  if (next === 'entry') entryKey.value++
  screen.value = next
}

function removeCard() {
  const current = card.value
  if (current?.answer) pendingDelete.value = { id: current.phrase_id, text: current.answer.text }
}

async function deletedCard() {
  pendingDelete.value = null
  await session.settleMedia()
  await practice.openDaily()
}

async function initialize() {
  initialized.value = false
  startupError.value = ''
  try {
    await resetDiscardedLearnerDataOnce()
    await takeTransfers.recover()
    initialized.value = true
  } catch (cause) {
    startupError.value = String(cause)
  }
}

onMounted(initialize)
</script>

<template>
  <div class="shell">
    <header>
      <a class="brand" href="/">Phrase Gym<span>◌</span></a>
      <nav v-if="initialized">
        <button @click="navigate('entry')">Practice</button
        ><button @click="navigate('settings')">Settings</button>
      </nav>
      <AddMaterial v-if="initialized" :before-open="session.leave" @added="entryKey++" />
    </header>
    <main>
      <CleanupNotice v-if="initialized" />
      <TakeRejections />
      <p v-if="startupError" class="error" role="alert">
        {{ startupError }} <button @click="initialize">Retry connection</button>
      </p>
      <p v-else-if="!initialized" role="status">Loading…</p>
      <SettingsPanel v-else-if="screen === 'settings'" />
      <template v-else-if="screen === 'card' && card">
        <p class="eyebrow">English</p>
        <StudyCard
          :card="card"
          :progress="practice.view.value.progress"
          :details-open="practice.view.value.detailsOpen"
          :capturing="capture.capturing.value"
          :requesting="capture.requesting.value"
          :processing="capture.processing.value"
          :playing="playback.active.value"
          :busy="practice.busy.value"
          :has-take="session.latestTake.value !== null"
          :pending-takes="takeTransfers.pendingTakes.value"
          :capture-error="capture.error.value"
          :take-error="takeTransfers.error.value"
          :practice-error="practice.error.value"
          :can-retry="practice.pending.value !== null"
          @control="session.controlAudio"
          @show="practice.reveal()"
          @details="card.revealed ? practice.toggleDetails() : practice.reveal(true)"
          @new-cue="practice.newCue"
          @hint="practice.hint"
          @complete="session.complete"
          @remove="removeCard"
          @play-phrase="card.answer && playback.play(card.answer.reference_url)"
          @play-mine="session.playMine"
          @compare="session.compareMine"
          @retry-save="takeTransfers.flush"
          @retry-completion="practice.retry"
        />
      </template>
      <template v-else>
        <p class="eyebrow">English</p>
        <PracticeEntry
          :key="entryKey"
          v-model:lesson-id="lessonId"
          :message="practice.view.value.message"
          :error="practice.error.value"
          :busy="practice.busy.value"
          :play="playback.play"
          :compare="playback.compare"
          :stop-playback="playback.stop"
          @daily="daily"
        />
      </template>
      <p v-if="playback.error.value" class="error" role="alert">{{ playback.error.value }}</p>
      <DeleteDialog :target="pendingDelete" @cancel="pendingDelete = null" @deleted="deletedCard" />
    </main>
  </div>
</template>
