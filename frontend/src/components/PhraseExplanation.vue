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
// Basic Coach cards carry no authored help, so they offer no explanation control.
const authored = computed(() => !!longerExplanation.value || props.usageContexts.length > 0)
</script>

<template>
  <div class="answer-help">
    <p v-if="explanation"><EmphasisText :text="explanation" /></p>
    <p v-if="usageContexts.length" class="purpose">Used for: {{ usageContexts.join(' · ') }}</p>
    <button
      v-if="authored"
      class="explain-link"
      :disabled="!longerExplanation"
      :aria-expanded="!!longerExplanation && open"
      @click="$emit('toggle')"
    >
      More explanation <span>{{ open ? '▴' : '▾' }}</span>
    </button>
    <div v-if="longerExplanation && open" class="explanation">
      <p><EmphasisText :text="longerExplanation" /></p>
    </div>
  </div>
</template>
