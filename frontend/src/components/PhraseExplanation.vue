<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  explanation: string
  usageContexts: string[]
  acceptedVariants: string[]
  detailedExplanation?: string | null
  open: boolean
}>()
defineEmits<{ toggle: [] }>()
const longerExplanation = computed(() => props.detailedExplanation?.trim())
</script>

<template>
  <div class="answer-help">
    <p>{{ explanation }}</p>
    <p v-if="usageContexts.length" class="purpose">Used for: {{ usageContexts.join(' · ') }}</p>
    <p v-if="acceptedVariants.length" class="muted">
      Also accepted: {{ acceptedVariants.join('; ') }}
    </p>
    <button
      :disabled="!longerExplanation"
      :aria-expanded="!!longerExplanation && open"
      @click="$emit('toggle')"
    >
      {{ longerExplanation && open ? 'Less explanation ▴' : 'More explanation ▾' }}
    </button>
    <div v-if="longerExplanation && open" class="explanation">
      <p>{{ longerExplanation }}</p>
    </div>
  </div>
</template>
