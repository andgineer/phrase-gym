<script setup lang="ts">
import type { HistoryItem, Recording } from '../studyTypes'
defineProps<{
  items: HistoryItem[]
  play: (url: string) => Promise<void>
  compare: (recording: Recording, referenceUrl: string) => Promise<void>
  disabled: boolean
}>()
</script>

<template>
  <section v-if="items.some((item) => item.recordings.length)" class="card history">
    <h2>Saved recording history</h2>
    <template v-for="item in items" :key="item.id">
      <div v-if="item.recordings.length" class="history-card">
        <h3>{{ item.answer }}</h3>
        <p class="source">{{ item.source_label }}</p>
        <div v-for="(recording, index) in item.recordings" :key="recording.id" class="take">
          <span>Take {{ index + 1 }}</span>
          <button :disabled="disabled" @click="play(recording.url)">Replay saved take</button>
          <button :disabled="disabled" @click="compare(recording, item.reference_url)">
            Compare saved take
          </button>
        </div>
      </div>
    </template>
  </section>
</template>
