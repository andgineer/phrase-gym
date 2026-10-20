<script setup lang="ts">
import type { usePracticeSession } from '../composables/usePracticeSession'
import { usePracticeActions } from '../composables/usePracticeActions'
import type { Language } from '../api'
import type { PracticeMessages } from '../practiceMessages'
import PracticeCard from './PracticeCard.vue'
import { serverUnavailable, connectionRetrying } from '../api'
const props = defineProps<{
  engine: ReturnType<typeof usePracticeSession>
  pt: PracticeMessages
  locale: Language
  nativeLanguage: Language
}>()
const emit = defineEmits<{ leave: [] }>()
const {
  session,
  item,
  card,
  error,
  transition,
  intro,
  currentStep,
  media,
  capturing,
  requesting,
  processing,
  captureError,
  playing,
  audioError,
  pendingTakes,
  takeError,
  latestUrl,
  ready,
  preparing,
  ratingBusy,
  help,
  rate,
  next,
  openIntake,
  pause,
  retry,
  beginDialogue,
  isolate,
} = usePracticeActions(props.engine)
defineExpose({ openIntake })
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
    <div v-if="session.status === 'completed' || session.status === 'superseded'" class="card">
      <h1 tabindex="-1">
        {{ session.status === 'completed' ? pt.complete : pt.results }}
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
                  : attempt.eligibility_reason === 'ungraded_study'
                    ? pt.study
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
      <button
        :disabled="
          engine.restoring.value ||
          preparing ||
          ratingBusy ||
          serverUnavailable ||
          connectionRetrying ||
          engine.pending.value.length > 0
        "
        @click="beginDialogue"
      >
        {{ pt.rehearse }}
      </button>
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
              :disabled="
                capturing ||
                requesting ||
                processing ||
                ratingBusy ||
                !ready ||
                engine.restoring.value ||
                serverUnavailable ||
                connectionRetrying
              "
              @click="media.audio.play(`/api/v1/media/${turn.audio_id}`)"
            >
              ▶ {{ pt.reference }}
            </button>
          </p>
        </template>
        <button
          :disabled="
            capturing ||
            requesting ||
            processing ||
            ratingBusy ||
            !ready ||
            engine.restoring.value ||
            serverUnavailable ||
            connectionRetrying
          "
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
          ratingBusy ||
          preparing ||
          !ready ||
          engine.restoring.value ||
          serverUnavailable ||
          connectionRetrying ||
          engine.pending.value.length > 0
        "
        :has-take="Boolean(latestUrl)"
        @control="media.control"
        @mine="media.audio.play(latestUrl)"
        @reference="help('reference')"
        @answer="help('answer')"
        @explain="help('explain')"
        @rate="rate"
        @next="next"
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
