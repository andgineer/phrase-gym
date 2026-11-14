<script setup lang="ts">
import { computed } from 'vue'
import EmphasisText from './EmphasisText.vue'

const props = defineProps<{
  explanation: string
  usageContexts: string[]
  detailedExplanation?: string | null
  open: boolean
}>()
defineEmits<{ toggle: [] }>()
const longerExplanation = computed(() => props.detailedExplanation?.trim())
</script>

<template>
  <div class="answer-help">
    <p><EmphasisText :text="explanation" /></p>
    <p v-if="usageContexts.length" class="purpose">Used for: {{ usageContexts.join(' · ') }}</p>
    <button
      :disabled="!longerExplanation"
      :aria-expanded="!!longerExplanation && open"
      @click="$emit('toggle')"
    >
      {{ longerExplanation && open ? 'Less explanation ▴' : 'More explanation ▾' }}
    </button>
    <div v-if="longerExplanation && open" class="explanation">
      <p><EmphasisText :text="longerExplanation" /></p>
    </div>
  </div>
</template>
