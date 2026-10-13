<script setup lang="ts">
import type { PracticeMessages } from '../practiceMessages'
defineProps<{ pt: PracticeMessages; busy: boolean }>()
defineEmits<{ retry: [] }>()
function stop() {
  window.dispatchEvent(new Event('phrase-gym:stop-media'))
}
</script>
<template>
  <section class="card" role="alert" data-testid="connection-required">
    <h1>{{ pt.connectionRequired }}</h1>
    <p>{{ pt.connectionRetry }}</p>
    <button :disabled="busy" @click="$emit('retry')">{{ pt.retry }}</button>
    <button @click="stop">{{ pt.stopOrCancel }}</button>
  </section>
</template>
