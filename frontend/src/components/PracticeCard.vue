<script setup lang="ts">
import { computed } from 'vue'
import type { CardState } from '../reducer'
import type { PlannedTarget, RecallRating } from '../practiceTypes'
import type { PracticeMessages } from '../practiceMessages'
import type { Language } from '../api'
const props = defineProps<{
  item: PlannedTarget
  card: CardState
  pt: PracticeMessages
  nativeLanguage: Language
  capturing: boolean
  requesting: boolean
  processing: boolean
  playing: boolean
  ratingBusy: boolean
  hasTake: boolean
}>()
const cue = computed(() => props.item.phrase.content.cues?.find((c) => c.id === props.item.cue_id))
const prompt = computed(() => cue.value?.prompt || props.item.phrase.content.purpose)
const showPhrase = computed(
  () =>
    props.card.revealed ||
    props.item.phrase.content.chunks.some((chunk) => cue.value?.visible_chunks.includes(chunk.id)),
)
const explanation = computed(() => props.item.phrase.content.explanations?.[props.nativeLanguage])
const emit = defineEmits<{
  control: []
  mine: []
  reference: []
  answer: []
  explain: []
  rate: [RecallRating]
}>()
</script>
<template>
  <article class="card practice-card">
    <p class="practice-prompt" :lang="item.phrase.language">
      {{ prompt }}
    </p>
    <h1 v-if="showPhrase" :lang="item.phrase.language" data-testid="practice-phrase">
      <template v-for="chunk in item.phrase.content.chunks" :key="chunk.id">
        <span v-if="card.revealed || cue?.visible_chunks.includes(chunk.id)">{{ chunk.text }}</span>
        <span v-else class="gap" aria-label="…">_____ </span>
      </template>
    </h1>
    <div class="audio-controls">
      <button
        class="microphone"
        :class="{ capturing }"
        :disabled="requesting || processing || ratingBusy"
        :aria-label="capturing ? pt.stopRecording : playing ? pt.stop : pt.record"
        @click="emit('control')"
      >
        <span aria-hidden="true">{{ capturing || playing ? '■' : '🎙' }}</span>
      </button>
      <button
        :disabled="!hasTake || capturing || requesting || processing || ratingBusy"
        @click="emit('mine')"
      >
        ▶ {{ pt.mine }}
      </button>
      <button
        :disabled="capturing || requesting || processing || ratingBusy"
        @click="emit('reference')"
      >
        ▶ {{ pt.reference }}
      </button>
    </div>
    <p v-if="capturing" role="status">{{ pt.stopHint }}</p>
    <div class="answer-controls">
      <button
        v-if="!card.revealed"
        :disabled="capturing || requesting || processing || ratingBusy"
        @click="emit('answer')"
      >
        {{ pt.answer }}
      </button>
      <button
        class="explain-link"
        :disabled="capturing || requesting || processing || ratingBusy"
        @click="emit('explain')"
      >
        📖 {{ pt.explain }}
      </button>
    </div>
    <div v-if="card.expanded" class="explanation" :lang="nativeLanguage">
      <template v-if="explanation">
        <p>{{ explanation.meaning }}</p>
        <p>{{ explanation.usage }}</p>
        <ul>
          <li v-for="chunk in explanation.breakdown" :key="chunk">
            {{ chunk }}
          </li>
        </ul>
      </template>
      <p v-else>{{ pt.missingExplanation }}</p>
    </div>
    <details>
      <summary>{{ pt.counts }}</summary>
      <p>{{ pt.criteria }}</p>
      <ul v-if="card.revealed">
        <li v-for="variant in item.phrase.content.acceptable_equivalents" :key="variant">
          {{ variant }}
        </li>
      </ul>
    </details>
    <p>{{ pt.rating }}</p>
    <div class="ratings">
      <button
        class="not-yet"
        :disabled="capturing || requesting || processing || ratingBusy"
        @click="emit('rate', 'not_yet')"
      >
        ↻ {{ pt.notYet }}
      </button>
      <button
        class="effort"
        :disabled="capturing || requesting || processing || ratingBusy"
        @click="emit('rate', 'with_effort')"
      >
        ◷ {{ pt.effort }}
      </button>
      <button
        class="recalled"
        :disabled="capturing || requesting || processing || ratingBusy"
        @click="emit('rate', 'recalled')"
      >
        ✓ {{ pt.recalled }}
      </button>
    </div>
    <nav class="lesson-links">
      <RouterLink
        v-for="lesson in item.phrase.lessons"
        :key="lesson.id"
        :to="`/learn/${item.phrase.language}/lessons/${lesson.id}`"
        >{{ lesson.title }} →</RouterLink
      >
    </nav>
  </article>
</template>
