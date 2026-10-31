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
import { useDatedMessage } from './learningDay'

const session = usePracticeSession()
const { practice, playback, capture } = session
const initialized = ref(false)
const startupError = ref('')
const screen = ref<'entry' | 'card' | 'settings'>('entry')
const lessonId = ref('')
const entryKey = ref(0)
const pendingDelete = ref<DeletionTarget | null>(null)
// Owned here so the entry, remounted after a finished Practice card, reopens its details.
const detailsPhrase = ref<string | null>(null)
const card = computed(() => practice.view.value.card)
const entryMessage = useDatedMessage(() => practice.view.value)

// An exhausted traversal returns to entry; a finished Practice card returns to its details.
watch(card, (current, previous) => {
  if (current || screen.value !== 'card') return
  if (previous?.mode === 'practice') detailsPhrase.value = previous.phrase_id
  screen.value = 'entry'
})

// Only this open's own card counts; a card left in the client is never shown again.
async function start(open: () => Promise<boolean>) {
  if (!(await open())) return
  detailsPhrase.value = null
  screen.value = 'card'
}

const daily = () => start(() => practice.openDaily(lessonId.value || null))
const extra = () => start(() => practice.openExtra(lessonId.value || null))
const retryExtra = () => start(practice.retryExtra)

async function practisePhrase(phraseId: string) {
  await session.settleMedia()
  await start(() => practice.openPhrase(phraseId))
}

async function navigate(next: 'entry' | 'settings') {
  await session.leave()
  void takeTransfers.flush()
  detailsPhrase.value = null
  if (next === 'entry') entryKey.value++
  screen.value = next
  // The entry never holds a card, so a failed open cannot fall back to a stale one.
  practice.closeCard()
}

function removeCard() {
  const current = card.value
  if (current?.answer) pendingDelete.value = { id: current.phrase_id, text: current.answer.text }
}

async function deletedCard() {
  pendingDelete.value = null
  await session.settleMedia()
  if (card.value?.mode !== 'practice') return practice.openDaily()
  // The deleted phrase has no details to return to.
  screen.value = 'entry'
  detailsPhrase.value = null
  practice.closeCard()
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
          :busy="practice.busy.value || session.completing.value"
          :has-take="session.latestTake.value !== null"
          :pending-takes="takeTransfers.uploadingTakes.value"
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
          @stop="navigate('entry')"
        />
      </template>
      <template v-else>
        <p class="eyebrow">English</p>
        <PracticeEntry
          :key="entryKey"
          v-model:lesson-id="lessonId"
          :message="entryMessage"
          :error="practice.error.value"
          :busy="practice.busy.value"
          :can-retry-extra="practice.pendingExtra.value !== null"
          v-model:details-phrase="detailsPhrase"
          :play="playback.play"
          :compare="playback.compare"
          :stop-playback="playback.stop"
          @daily="daily"
          @extra="extra"
          @retry-extra="retryExtra"
          @practice="practisePhrase"
        />
      </template>
      <p v-if="playback.error.value" class="error" role="alert">{{ playback.error.value }}</p>
      <DeleteDialog :target="pendingDelete" @cancel="pendingDelete = null" @deleted="deletedCard" />
    </main>
  </div>
</template>
