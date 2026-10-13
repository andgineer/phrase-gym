<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { usePracticeSession } from '../composables/usePracticeSession'
import { useCardAudio } from '../composables/useCardAudio'
import type { Language } from '../api'
import type { RecallRating } from '../practiceTypes'
import type { PracticeMessages } from '../practiceMessages'
import PracticeCard from './PracticeCard.vue'
import SessionDownload from './SessionDownload.vue'
const props = defineProps<{
  engine: ReturnType<typeof usePracticeSession>
  pt: PracticeMessages
  nativeLanguage: Language
}>()
const emit = defineEmits<{ leave: [] }>()
const { session, item, card, error, present, action, flush, transition } = props.engine
const ratingBusy = ref(false)
const intro = computed(() =>
  Boolean(session.value?.manifest.dialogue && !session.value.queue[0]?.introduced),
)
const currentStep = computed(() => session.value?.queue[session.value.cursor])
let captureContext: { sessionId: string; step: number } | null = null
const media = useCardAudio(
  async () => {
    if (session.value) captureContext = { sessionId: session.value.id, step: session.value.cursor }
    const captureEvent = await action('respond', { modality: 'recorded' })
    const attempt = session.value?.attempts.find((a) => a.step === session.value?.cursor)
    if (!attempt || !item.value) throw new Error('Attempt is not ready')
    return {
      target_id: item.value.phrase.id,
      revision: item.value.phrase.revision,
      attempt_id: attempt.id,
      capture_event_id: captureEvent.id,
    }
  },
  () => `/api/v1/media/${item.value?.phrase.content.reference_audio_id}`,
  async () => {
    if (
      session.value?.id === captureContext?.sessionId &&
      session.value?.cursor === captureContext?.step
    )
      await action('failure', { reason: 'Capture unavailable or interrupted' })
  },
  async () => {
    if (
      session.value?.id === captureContext?.sessionId &&
      session.value?.cursor === captureContext?.step
    )
      await action('capture_complete')
  },
)
const { capturing, requesting, processing, error: captureError } = media.capture
const { playing, error: audioError } = media.audio
const { pendingTakes, error: takeError } = media.takes
const { latestUrl } = media
const ready = computed(() => session.value?.attempts.some((a) => a.step === session.value?.cursor))
async function prepare() {
  if (session.value?.status !== 'active') {
    await media.release()
    return
  }
  if (!item.value) return
  if (intro.value) {
    try {
      await action('dialogue_exposure')
    } catch {}
    return
  }
  latestUrl.value = ''
  try {
    await present()
    await media.refresh(item.value.phrase.id)
  } catch {}
}
watch(
  [() => session.value?.id, () => session.value?.cursor, () => session.value?.status, intro],
  prepare,
  {
    immediate: true,
  },
)
async function help(kind: 'answer' | 'explain' | 'reference') {
  const origin = session.value
  const reference = `/api/v1/media/${item.value?.phrase.content.reference_audio_id}`
  media.audio.stop()
  try {
    await action(kind)
    if (
      kind === 'reference' &&
      session.value?.id === origin?.id &&
      session.value?.cursor === origin?.cursor
    )
      void media.audio.play(reference)
  } catch {}
}
async function rate(rating: RecallRating) {
  if (ratingBusy.value) return
  ratingBusy.value = true
  media.audio.stop()
  try {
    await action('rate', { rating })
    void media.takes.flush()
  } catch {
  } finally {
    ratingBusy.value = false
  }
}
async function pause() {
  await media.release()
  await transition('paused')
}
async function retry() {
  try {
    await props.engine.load()
    await flush()
    await prepare()
    await media.takes.flush()
    if (audioError.value) void media.audio.retry()
    else if (captureError.value) void media.capture.start()
  } catch {}
}
async function beginDialogue() {
  try {
    await action('dialogue_exposure')
    await action('dialogue_begin')
  } catch {}
}
async function isolate() {
  try {
    await action('detour')
  } catch {}
}
</script>
<template>
  <section v-if="session" class="practice-panel" :aria-busy="engine.restoring.value">
    <div class="session-toolbar">
      <span
        >{{ Math.min(session.cursor + 1, session.queue.length) }} / {{ session.queue.length }}</span
      >
      <button
        v-if="session.status === 'active'"
        :disabled="engine.busy.value || engine.restoring.value"
        @click="pause"
      >
        {{ pt.pause }}
      </button>
      <button @click="emit('leave')">{{ pt.back }}</button>
    </div>
    <SessionDownload
      :engine="engine"
      :pt="pt"
      :capture-busy="capturing || requesting || processing"
      :pending-takes="pendingTakes"
      @retry="retry"
    />
    <div v-if="session.status === 'completed' || session.status === 'superseded'" class="card">
      <h1 tabindex="-1">
        {{
          session.status === 'completed'
            ? engine.pending.value.length
              ? pt.completePending
              : pt.complete
            : pt.results
        }}
      </h1>
      <div v-if="session.manifest.dialogue" :lang="session.manifest.language">
        <p v-for="(turn, index) in session.manifest.dialogue.content.turns" :key="index">
          <strong>{{ turn.speaker }}:</strong> {{ turn.text }}
        </p>
      </div>
      <h2>{{ pt.results }}</h2>
      <ul>
        <li v-for="attempt in session.attempts" :key="attempt.id">
          {{
            session.manifest.targets.find((t) => t.phrase.id === attempt.target_id)?.phrase.content
              .text
          }}
          —
          {{
            attempt.rating === 'recalled'
              ? pt.recalled
              : attempt.rating === 'with_effort'
                ? pt.effort
                : attempt.rating === 'not_yet'
                  ? pt.notYet
                  : pt.reported
          }}
          <span v-if="attempt.original?.supported"> · {{ pt.supported }}</span>
        </li>
      </ul>
    </div>
    <div v-else-if="session.status === 'paused'" class="card">
      <h1>{{ pt.paused }}</h1>
      <button :disabled="engine.busy.value || engine.restoring.value" @click="transition('active')">
        {{ pt.resume }}
      </button>
    </div>
    <div v-else-if="intro && session.manifest.dialogue" class="card">
      <h1>{{ session.manifest.dialogue.content.title }}</h1>
      <p
        v-for="(turn, index) in session.manifest.dialogue.content.turns"
        :key="index"
        :lang="session.manifest.language"
      >
        <strong>{{ turn.speaker }}:</strong> {{ turn.text }}
      </p>
      <button @click="beginDialogue">{{ pt.rehearse }}</button>
    </div>
    <template v-else-if="session.status === 'active' && item && card">
      <section
        v-if="session.manifest.dialogue && currentStep?.dialogue_turn != null"
        class="dialogue-lines"
        :lang="session.manifest.language"
      >
        <template v-for="(turn, index) in session.manifest.dialogue.content.turns" :key="index">
          <p v-if="index < currentStep.dialogue_turn">
            <strong>{{ turn.speaker }}:</strong> {{ turn.text }}
            <button
              v-if="turn.audio_id"
              :disabled="capturing || requesting || processing || ratingBusy || !ready"
              @click="media.audio.play(`/api/v1/media/${turn.audio_id}`)"
            >
              ▶ {{ pt.reference }}
            </button>
          </p>
        </template>
        <button
          :disabled="capturing || requesting || processing || ratingBusy || !ready"
          @click="isolate"
        >
          {{ pt.isolate }}
        </button>
      </section>
      <PracticeCard
        :key="session.cursor"
        :item="item"
        :card="card"
        :pt="pt"
        :native-language="nativeLanguage"
        :capturing="capturing"
        :requesting="requesting"
        :processing="processing"
        :playing="playing"
        :rating-busy="
          ratingBusy || !ready || engine.restoring.value || engine.offlineUnavailable.value
        "
        :has-take="Boolean(latestUrl)"
        @control="media.control"
        @mine="media.audio.play(latestUrl)"
        @reference="help('reference')"
        @answer="help('answer')"
        @explain="help('explain')"
        @rate="rate"
      />
    </template>
    <p v-if="engine.busy.value" role="status">{{ pt.syncing }}</p>
    <p v-if="pendingTakes" role="status">{{ pt.pending }} ({{ pendingTakes }})</p>
    <p v-if="error || captureError || audioError || takeError" class="error" role="alert">
      {{ error || captureError || audioError || takeError }}
      <button @click="retry">{{ pt.retry }}</button>
    </p>
  </section>
</template>
