<script setup lang="ts">
defineProps<{ disabled: boolean; otherCue: boolean; hint: boolean }>()
defineEmits<{ hint: []; newCue: []; reveal: [] }>()
</script>
<template>
  <div class="card-controls front-controls">
    <button class="tinted" :disabled="disabled || !hint" @click="$emit('hint')">
      <svg
        class="audio-icon"
        aria-hidden="true"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M9 18h6m-5 3h4M8 14a6 6 0 1 1 8 0l-1 3H9z" />
      </svg>
      Hint
    </button>
    <button class="tinted" :disabled="disabled || !otherCue" @click="$emit('newCue')">
      <span aria-hidden="true">↻</span> New cue
    </button>
    <button class="primary" :disabled="disabled" @click="$emit('reveal')">
      <svg
        class="audio-icon"
        aria-hidden="true"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
      >
        <path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12Z" />
        <circle cx="12" cy="12" r="3" />
      </svg>
      Reveal
    </button>
  </div>
</template>
