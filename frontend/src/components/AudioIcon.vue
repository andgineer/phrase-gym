<script setup lang="ts">
defineProps<{ name: 'reference' | 'mine' | 'history' | 'compare' }>()
const paths = {
  reference: 'M11 5 6 9H3v6h3l5 4V5M15 8a6 6 0 0 1 0 8M18 5a10 10 0 0 1 0 14',
  mine: 'M20 21v-2a7 7 0 0 0-14 0v2M17 7a4 4 0 1 1-8 0 4 4 0 0 1 8 0',
  history: 'M3 11a9 9 0 1 1 2.6 7.4M3 4v7h7M12 7v5l3 2',
  compare: 'M4 7h16l-4-4M20 17H4l4 4',
}
</script>

<template>
  <svg
    class="audio-icon"
    aria-hidden="true"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path :d="paths[name]" />
  </svg>
</template>
