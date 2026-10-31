<script setup lang="ts">
import { computed } from 'vue'
import { usePracticeFacts } from '../composables/usePracticeFacts'
import {
  bestLine,
  counted,
  nextReturnLine,
  ratingsLine,
  streakLine,
  studiedLine,
  todayLine,
  weekday,
  workLine,
} from '../practiceFacts'

const props = defineProps<{ message: string }>()
const { facts, error, listing, studied, uploadingTakes, refresh, toggleList } = usePracticeFacts()
// Facts of this mount confirm the learning date before an earlier message can be judged current.
const currentMessage = computed(() => (facts.value || error.value ? props.message : ''))
const today = computed(() => facts.value?.today)
const practised = computed(() => (facts.value?.work.completions ?? 0) > 0)
const todayShown = computed(
  () => !!today.value && (today.value.completions || today.value.recordings),
)
const visible = computed(
  () => currentMessage.value || practised.value || uploadingTakes.value || error.value,
)
defineExpose({ refresh })
</script>

<template>
  <section v-if="visible" class="card practice-facts" aria-label="Practice facts">
    <h2 v-if="today?.completions">Today’s practice ✦</h2>
    <p v-if="currentMessage" role="status" class="entry-message">{{ currentMessage }}</p>
    <template v-if="facts && practised">
      <div class="streak">
        <p data-testid="streak">{{ streakLine(facts.streak) }}</p>
        <p class="muted" data-testid="best-streak">{{ bestLine(facts.streak) }}</p>
      </div>
      <ol class="recent-days" aria-label="Last seven days" data-testid="recent-days">
        <li
          v-for="day in facts.streak.recent_days"
          :key="day.learning_date"
          :class="{ practiced: day.practiced }"
          :aria-label="`${weekday(day.learning_date)}: ${day.practiced ? 'practiced' : 'no practice'}`"
        >
          {{ weekday(day.learning_date) }}
          <span aria-hidden="true">{{ day.practiced ? '●' : '○' }}</span>
        </li>
      </ol>
    </template>
    <div v-if="today && todayShown" class="facts-group">
      <h3 class="eyebrow">Today · all lessons</h3>
      <p v-if="today.completions" data-testid="today-counts">{{ todayLine(today) }}</p>
      <p v-if="ratingsLine(today)" data-testid="today-ratings">{{ ratingsLine(today) }}</p>
      <p v-if="today.recordings" data-testid="today-recordings">
        {{ counted(today.recordings, 'recording') }} available
      </p>
      <button v-if="today.completions" :aria-expanded="listing" @click="toggleList">
        See today’s phrases
      </button>
      <template v-if="listing">
        <ul class="phrase-list studied-list" aria-label="Today’s phrases">
          <li v-for="item in studied.items.value" :key="item.completion_id">
            <strong lang="en">{{ item.text }}</strong>
            <span class="muted">{{ studiedLine(item) }}</span>
          </li>
        </ul>
        <button v-if="studied.nextOffset.value !== null" @click="studied.more">More</button>
        <p v-if="studied.error.value" role="alert" class="error">{{ studied.error.value }}</p>
      </template>
    </div>
    <p v-if="uploadingTakes" class="muted" data-testid="pending-uploads">
      {{ counted(uploadingTakes, 'recording') }} waiting to upload from this device
    </p>
    <div v-if="facts && practised" class="facts-group">
      <h3 class="eyebrow">Your work · all lessons</h3>
      <p data-testid="your-work">{{ workLine(facts) }}</p>
      <p v-if="facts.next_return" data-testid="next-return">{{ nextReturnLine(facts) }}</p>
    </div>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
  </section>
</template>
