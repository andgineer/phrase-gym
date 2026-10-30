<script setup lang="ts">
defineProps<{ capturing: boolean; requesting: boolean; playing: boolean; disabled: boolean }>()
const emit = defineEmits<{ control: [] }>()
</script>

<template>
  <button
    class="microphone"
    :class="{ capturing }"
    :disabled="disabled"
    :aria-label="
      requesting
        ? 'Cancel microphone request'
        : capturing
          ? 'Stop recording'
          : playing
            ? 'Stop playback'
            : 'Record a take'
    "
    @click="emit('control')"
  >
    <span v-if="capturing || requesting || playing" aria-hidden="true" class="stop-symbol" />
    <svg
      v-else
      aria-hidden="true"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <rect x="9" y="2" width="6" height="12" rx="3" />
      <path d="M5 10a7 7 0 0 0 14 0M12 17v5m-4 0h8" />
    </svg>
  </button>
</template>
