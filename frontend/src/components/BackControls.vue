<script setup lang="ts">
import AudioIcon from './AudioIcon.vue'
defineProps<{ disabled: boolean; hasTake: boolean; historyOpen: boolean }>()
defineEmits<{ reference: []; mine: []; compare: []; history: [] }>()
</script>
<template>
  <div class="card-controls back-controls">
    <button :disabled="disabled" @click="$emit('reference')">
      <AudioIcon name="reference" /> Ref
    </button>
    <button :disabled="disabled || !hasTake" @click="$emit('mine')">
      <AudioIcon name="mine" /> Mine
    </button>
    <button :disabled="disabled || !hasTake" @click="$emit('compare')">
      <AudioIcon name="compare" /> Compare
    </button>
    <button aria-label="Recording history" :aria-expanded="historyOpen" @click="$emit('history')">
      <AudioIcon name="history" /><span aria-hidden="true">{{ historyOpen ? '▴' : '▾' }}</span>
    </button>
  </div>
</template>
