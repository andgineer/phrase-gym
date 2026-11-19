<script setup lang="ts">
import { returnDescription, returnSpan } from '../returnSpan'
import type { Card, Outcome, Rating } from '../studyTypes'

defineProps<{ disabled: boolean; returnDays?: Card['return_days']; selected?: Outcome }>()
defineEmits<{ complete: [outcome: Outcome] }>()

const ratings: Array<[Rating, string, string]> = [
  ['not_yet', 'Again', 'not-yet'],
  ['effort', 'Hard', 'effort'],
  ['got_it', 'Good', 'recalled'],
  ['easy', 'Easy', 'recalled'],
]
</script>

<template>
  <button
    v-for="[outcome, label, style] in ratings"
    :key="outcome"
    :class="style"
    :title="returnDays ? returnDescription(returnDays[outcome]) : undefined"
    :aria-pressed="selected === undefined ? undefined : selected === outcome"
    :disabled="disabled"
    @click="$emit('complete', outcome)"
  >
    <span v-if="returnDays" class="rating-return" aria-hidden="true">{{
      returnSpan(returnDays[outcome])
    }}</span
    >{{ label }}
  </button>
  <button
    class="rating-navigation"
    aria-label="Skip"
    title="Skip"
    :aria-pressed="selected === undefined ? undefined : selected === 'skipped'"
    :disabled="disabled"
    @click="$emit('complete', 'skipped')"
  >
    <span class="rating-chevron" aria-hidden="true" />
  </button>
</template>
