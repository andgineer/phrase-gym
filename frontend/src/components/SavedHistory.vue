<script setup lang="ts">
import { ref } from 'vue'
import AudioIcon from './AudioIcon.vue'
import SavedHistoryPanel from './SavedHistoryPanel.vue'
defineProps<{ phraseId: string; disabled: boolean; play: (url: string) => Promise<void> }>()
const expanded = ref(false)
</script>
<template>
  <div class="saved-history">
    <button
      aria-label="Recording history"
      title="Recording history"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <AudioIcon name="history" /> <span aria-hidden="true">{{ expanded ? '▴' : '▾' }}</span>
    </button>
    <SavedHistoryPanel v-if="expanded" :phrase-id="phraseId" :disabled="disabled" :play="play" />
  </div>
</template>
