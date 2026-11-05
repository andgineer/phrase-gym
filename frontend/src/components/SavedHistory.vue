<script setup lang="ts">
import { ref, watch } from 'vue'
import AudioIcon from './AudioIcon.vue'
import { usePagedList } from '../composables/usePagedList'
import { takeTransfers } from '../takeTransfers'
import type { Recording, RecordingPage } from '../studyTypes'

const props = defineProps<{
  phraseId: string
  disabled: boolean
  play: (url: string) => Promise<void>
}>()
const takes = usePagedList<Recording, RecordingPage>(
  () => `/phrases/${encodeURIComponent(props.phraseId)}/recordings`,
  10,
)
const expanded = ref(false)
watch([expanded, takeTransfers.uploadingTakes], () => {
  if (expanded.value) void takes.refresh()
})
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
    <section v-if="expanded" class="recording-history-panel" aria-label="Saved recordings">
      <ul v-if="takes.items.value.length" class="take-history">
        <li v-for="take in takes.items.value" :key="take.id" class="take">
          <time :datetime="take.learning_date">{{ take.learning_date }}</time>
          <button :disabled="disabled" @click="play(take.url)">
            <AudioIcon name="mine" /> Mine
          </button>
        </li>
      </ul>
      <p v-else-if="takes.loaded.value" class="muted">No saved recordings yet.</p>
      <button v-if="takes.nextOffset.value !== null" @click="takes.more">More</button>
      <p v-if="takes.error.value" role="alert" class="error">{{ takes.error.value }}</p>
    </section>
  </div>
</template>
