<script setup lang="ts">
import { ref, watch } from 'vue'
import { usePagedList } from '../composables/usePagedList'
import { takeTransfers } from '../takeTransfers'
import type { Recording, RecordingPage } from '../studyTypes'

const props = defineProps<{
  phraseId: string
  referenceUrl: string
  disabled: boolean
  play: (url: string) => Promise<void>
  compare: (recording: Recording, referenceUrl: string) => Promise<void>
}>()
const modes: Record<Recording['mode'], string> = {
  daily: 'Daily',
  extra: 'Extra',
  practice: 'Practice',
}
const kinds: Record<Recording['kind'], string> = {
  first_study: 'Historical first meeting',
  new: 'New',
  recall: 'Recall',
}
const takes = usePagedList<Recording, RecordingPage>(
  () => `/phrases/${encodeURIComponent(props.phraseId)}/recordings`,
  10,
)
const expanded = ref(false)
watch([expanded, takeTransfers.uploadingTakes], () => {
  if (expanded.value) void takes.refresh()
})

function describe(take: Recording) {
  const parts = [take.learning_date, modes[take.mode], kinds[take.kind]]
  if (take.cue) parts.push(`Cue: ${take.cue.situation}`)
  if (take.hint) parts.push(`Hint: ${take.hint}`)
  return parts.join(' · ')
}
</script>

<template>
  <div class="saved-history">
    <button :aria-expanded="expanded" @click="expanded = !expanded">Recording history</button>
    <section v-if="expanded" aria-labelledby="saved-history-heading">
      <h2 id="saved-history-heading">Saved recordings</h2>
      <ul v-if="takes.items.value.length" class="take-history">
        <li v-for="take in takes.items.value" :key="take.id" class="take">
          <p>{{ describe(take) }}</p>
          <p class="muted">Answer then: {{ take.answer }}</p>
          <button :disabled="disabled" @click="play(take.url)">▶ Mine</button>
          <button :disabled="disabled" @click="compare(take, referenceUrl)">Compare</button>
        </li>
      </ul>
      <p v-else-if="takes.loaded.value" class="muted">No saved recordings yet.</p>
      <button v-if="takes.nextOffset.value !== null" @click="takes.more">More</button>
      <p v-if="takes.error.value" role="alert" class="error">{{ takes.error.value }}</p>
    </section>
  </div>
</template>
