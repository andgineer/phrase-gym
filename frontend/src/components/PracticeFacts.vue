<script setup lang="ts">
import { computed } from 'vue'
import { usePracticeFacts } from '../composables/usePracticeFacts'
import {
  bestLine,
  counted,
  libraryLine,
  progressLine,
  progressTitle,
  ratingsLine,
  recordingsLine,
  streakLine,
  studiedLine,
  todayTitle,
  totalTitle,
} from '../practiceFacts'

const props = defineProps<{ message: string }>()
const { facts, error, listing, studied, uploadingTakes, refresh, toggleList, dismiss } =
  usePracticeFacts()
// Facts of this mount confirm the learning date before an earlier message can be judged current.
const currentMessage = computed(() => (facts.value || error.value ? props.message : ''))
const today = computed(() => facts.value?.today)
const practised = computed(() => (facts.value?.work.completions ?? 0) > 0)
const todayShown = computed(
  () => !!today.value && (today.value.completions || today.value.recordings),
)
const notices = computed(() => facts.value?.notices ?? [])
const visible = computed(
  () =>
    currentMessage.value ||
    practised.value ||
    uploadingTakes.value ||
    error.value ||
    notices.value.length,
)
defineExpose({ refresh })
</script>

<template>
  <section v-if="visible" class="card practice-facts" aria-label="Practice facts">
    <p v-if="currentMessage" role="status" class="entry-message">{{ currentMessage }}</p>
    <template v-if="facts && practised">
      <div class="streak">
        <p data-testid="streak">{{ streakLine(facts.streak) }}</p>
        <p class="muted" data-testid="best-streak">{{ bestLine(facts.streak) }}</p>
      </div>
    </template>
    <div v-if="today && todayShown" class="facts-group">
      <h3 class="eyebrow" data-testid="today-title">{{ todayTitle(today) }}</h3>
      <p v-if="ratingsLine(today)" data-testid="today-ratings">{{ ratingsLine(today) }}</p>
      <p v-if="today.recordings" data-testid="today-recordings">
        {{ recordingsLine(today.recordings) }}
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
    <template v-if="facts && practised">
      <div class="facts-group">
        <h3 class="eyebrow" data-testid="progress-title">{{ progressTitle(facts) }}</h3>
        <p v-if="progressLine(facts)" data-testid="progress">{{ progressLine(facts) }}</p>
        <p v-if="ratingsLine(facts.work)" data-testid="progress-ratings">
          {{ ratingsLine(facts.work) }}
        </p>
        <p v-if="facts.work.recordings" data-testid="progress-recordings">
          {{ recordingsLine(facts.work.recordings) }}
        </p>
      </div>
      <div class="facts-group">
        <h3 class="eyebrow" data-testid="total-title">{{ totalTitle(facts.library) }}</h3>
        <p v-if="libraryLine(facts.library)" data-testid="library">
          {{ libraryLine(facts.library) }}
        </p>
        <p v-if="facts.library.long_term_count" data-testid="long-term">
          {{ facts.library.long_term_count }} in long-term memory
        </p>
      </div>
    </template>
    <ul v-if="notices.length" class="import-notices" aria-label="Import problems">
      <li v-for="notice in notices" :key="notice.package_id">
        <p>
          Coach could not prepare <strong>{{ notice.source_label }}</strong> ({{
            notice.package_id
          }}): {{ notice.error }}
        </p>
        <p class="muted">Import the same material again to retry.</p>
        <button
          class="quiet"
          :aria-label="`Dismiss notice about ${notice.package_id}`"
          @click="dismiss(notice.package_id)"
        >
          ×
        </button>
      </li>
    </ul>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
  </section>
</template>
