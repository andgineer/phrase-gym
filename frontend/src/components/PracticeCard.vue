<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { CardState } from '../practiceState'
import type { PlannedTarget, RecallRating } from '../practiceTypes'
import type { PracticeMessages } from '../practiceMessages'
import type { Language } from '../api'
import { preparedExplanation } from '../explanation'
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
const heading = ref<HTMLElement | null>(null)
watch(heading, (element) => element?.focus())
const study = computed(() => props.item.eligibility === 'study')
const english = computed(() => props.item.phrase.english)
const cue = computed(() => props.item.phrase.content.cues?.find((c) => c.id === props.item.cue_id))
const situation = computed(() => english.value?.situations.find((s) => s.id === props.item.cue_id))
const prompt = computed(
  () => situation.value?.text || cue.value?.prompt || props.item.phrase.content.purpose,
)
const presentedPrompt = computed(() => {
  if (english.value?.task?.kind !== 'word_sense') return { sentence: prompt.value, cue: '' }
  const boundary = prompt.value.indexOf(' | ')
  return boundary < 0
    ? { sentence: prompt.value, cue: '' }
    : { sentence: prompt.value.slice(0, boundary), cue: prompt.value.slice(boundary + 3) }
})
const showPhrase = computed(
  () =>
    props.card.revealed ||
    props.item.phrase.content.chunks.some((chunk) => cue.value?.visible_chunks.includes(chunk.id)),
)
const explanation = computed(() =>
  english.value?.help
    ? { ...english.value.help, breakdown: [] }
    : preparedExplanation(props.item.phrase.content.explanations?.[props.nativeLanguage]),
)
const helpLanguage = computed(() => english.value?.help?.language ?? props.nativeLanguage)
const emit = defineEmits<{
  control: []
  mine: []
  reference: []
  answer: []
  explain: []
  next: []
  rate: [RecallRating]
}>()
</script>
<template>
  <article class="card practice-card">
    <p v-if="study" role="status">{{ pt.study }}. {{ pt.studyHelp }}</p>
    <p class="practice-prompt" :lang="situation?.language || item.phrase.language">
      {{ presentedPrompt.sentence }}
    </p>
    <p v-if="presentedPrompt.cue" class="word-sense-cue" :lang="situation?.language">
      {{ presentedPrompt.cue }}
    </p>
    <p v-if="situation?.partner">{{ situation.partner.text }}</p>
    <h1
      v-if="showPhrase"
      ref="heading"
      tabindex="-1"
      :lang="item.phrase.language"
      data-testid="practice-phrase"
    >
      <template v-for="chunk in item.phrase.content.chunks" :key="chunk.id">
        <span v-if="card.revealed || cue?.visible_chunks.includes(chunk.id)">{{ chunk.text }}</span>
        <span v-else class="gap" aria-label="…">_____ </span>
      </template>
    </h1>
    <p v-if="english?.task?.kind === 'word_sense'">
      Say the whole short sentence. Rate the missing word and sense; an unrelated slip does not fail
      the word.
    </p>
    <p v-if="english?.task?.kind === 'frame'">
      Use your practiced construction; its variable slots may change.
    </p>
    <div v-if="card.revealed && english?.task" class="target-task">
      <template v-if="english.task.kind === 'frame'">
        <p>{{ english.task.pattern }}</p>
        <p v-for="slot in english.task.slots" :key="slot.name">
          {{ slot.name }}: {{ slot.values.join(' / ') }}
        </p>
        <p>Use this construction; the named slots may vary.</p>
      </template>
      <template v-else>
        <p>{{ english.task.word }} — {{ english.task.sense }}</p>
        <p>
          Say the whole sentence. Rate the selected word and sense; an unrelated slip does not fail
          the word.
        </p>
      </template>
    </div>
    <div class="audio-controls">
      <button
        class="microphone"
        :class="{ capturing }"
        :disabled="!(capturing || playing || requesting) && (processing || ratingBusy)"
        :aria-label="
          requesting
            ? pt.cancelPermission
            : capturing
              ? pt.stopRecording
              : playing
                ? pt.stop
                : pt.record
        "
        @click="emit('control')"
      >
        <span aria-hidden="true">{{ capturing || playing || requesting ? '■' : '🎙' }}</span>
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
        v-if="explanation"
        class="explain-link"
        :disabled="capturing || requesting || processing || ratingBusy"
        @click="emit('explain')"
      >
        📖 {{ pt.explain }}
      </button>
    </div>
    <div v-if="study && english?.help" :lang="english.help.language">
      <p>{{ english.help.meaning }}</p>
      <p v-if="english.help.usage">{{ english.help.usage }}</p>
    </div>
    <div v-else-if="card.expanded && explanation" class="explanation" :lang="helpLanguage">
      <p v-if="explanation.meaning.trim()">{{ explanation.meaning }}</p>
      <p v-if="explanation.usage?.trim()">{{ explanation.usage }}</p>
      <ul v-if="explanation.breakdown?.length">
        <li v-for="chunk in explanation.breakdown" :key="chunk">
          {{ chunk }}
        </li>
      </ul>
    </div>
    <p v-if="card.revealed && english?.help?.translation" class="answer-translation">
      {{ english.help.translation }}
    </p>
    <template v-if="!study">
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
      <p
        v-if="
          card.originalResponse?.supported || (!card.originalResponse && card.exposedBeforeResponse)
        "
        role="status"
      >
        {{ pt.supported }}
      </p>
      <div class="ratings">
        <button
          class="not-yet"
          :disabled="
            (english && !card.revealed) || capturing || requesting || processing || ratingBusy
          "
          @click="emit('rate', 'not_yet')"
        >
          ↻ {{ pt.notYet }}
        </button>
        <button
          class="effort"
          :disabled="
            (english && !card.revealed) || capturing || requesting || processing || ratingBusy
          "
          @click="emit('rate', 'with_effort')"
        >
          ◷ {{ pt.effort }}
        </button>
        <button
          class="recalled"
          :disabled="
            (english && !card.revealed) || capturing || requesting || processing || ratingBusy
          "
          @click="emit('rate', 'recalled')"
        >
          ✓ {{ pt.recalled }}
        </button>
      </div>
    </template>
    <button
      v-else
      class="primary"
      :disabled="capturing || requesting || processing || ratingBusy"
      @click="emit('next')"
    >
      {{ pt.next }}
    </button>
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
