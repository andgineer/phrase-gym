<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, useId } from 'vue'
import { useSavedRecordings } from '../composables/useSavedRecordings'
import type { Recording } from '../studyTypes'

const props = defineProps<{
  phraseId: string
  playingUrl: string | null
  playRecording: (take: Recording) => Promise<void>
  stopPlayback: () => void
}>()
const emit = defineEmits<{ back: [] }>()
const titleId = useId()
const back = ref<HTMLButtonElement | null>(null)
const { items, loaded, loading, error, nextOffset, more, refresh } = useSavedRecordings(
  () => props.phraseId,
)
const dayFormat = new Intl.DateTimeFormat(undefined, { dateStyle: 'long' })
const timeFormat = new Intl.DateTimeFormat(undefined, { timeStyle: 'short' })
const days = computed(() => {
  const today = dayFormat.format(new Date())
  const groups = new Map<string, { take: Recording; time: string }[]>()
  for (const take of items.value) {
    const captured = new Date(take.captured_at)
    const date = dayFormat.format(captured)
    const label = date === today ? 'Today' : date
    const group = groups.get(label) ?? []
    group.push({ take, time: timeFormat.format(captured) })
    groups.set(label, group)
  }
  return groups
})

function duration(milliseconds: number) {
  const seconds = Math.round(milliseconds / 1000)
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`
}
function toggle(take: Recording) {
  if (props.playingUrl === take.url) props.stopPlayback()
  else void props.playRecording(take)
}
function escape(event: KeyboardEvent) {
  if (event.key === 'Escape') emit('back')
}
onMounted(() => {
  back.value?.focus()
  document.addEventListener('keydown', escape)
})
onBeforeUnmount(() => {
  document.removeEventListener('keydown', escape)
  props.stopPlayback()
})
</script>

<template>
  <section class="recording-screen" :aria-labelledby="titleId">
    <button ref="back" class="quiet back" @click="emit('back')">
      <span aria-hidden="true">←</span> Back to practice
    </button>
    <h1 :id="titleId">Recording history</h1>
    <p v-if="!loaded && loading" role="status" class="muted">Loading recordings…</p>
    <p v-else-if="loaded && !items.length" class="muted">No saved recordings yet.</p>
    <section v-for="[day, rows] in days" :key="day" class="recording-day">
      <h2>{{ day }}</h2>
      <ul>
        <li v-for="{ take, time } in rows" :key="take.id">
          <button
            class="recording-row"
            :class="{ playing: playingUrl === take.url }"
            @click="toggle(take)"
          >
            <span class="recording-symbol" aria-hidden="true" />
            <span class="visually-hidden"
              >{{ playingUrl === take.url ? 'Stop' : 'Play' }} recording at</span
            >
            <time :datetime="take.captured_at">{{ time }}</time>
            <span v-if="take.duration_ms !== null" class="muted">{{
              duration(take.duration_ms)
            }}</span>
          </button>
        </li>
      </ul>
    </section>
    <p v-if="error" class="error" role="alert">
      {{ error }} <button @click="refresh">Retry</button>
    </p>
    <button v-if="nextOffset !== null" :disabled="loading" @click="more">
      Show older recordings
    </button>
  </section>
</template>

<style scoped>
.back {
  padding-inline: 0;
  margin: 0 0 4px;
}
.recording-day {
  margin: 24px 0;
}
.recording-day h2 {
  font-size: 0.95rem;
  margin: 0 0 8px;
}
.recording-day ul {
  padding: 0;
  margin: 0;
  list-style: none;
}
.recording-row {
  display: flex;
  align-items: center;
  width: 100%;
  gap: 12px;
  padding: 12px 16px;
  font-variant-numeric: tabular-nums;
  border-radius: 0;
}
li + li .recording-row {
  border-top: 0;
}
li:first-child .recording-row {
  border-top-left-radius: 12px;
  border-top-right-radius: 12px;
}
li:last-child .recording-row {
  border-bottom-left-radius: 12px;
  border-bottom-right-radius: 12px;
}
.recording-row time {
  margin-right: auto;
}
.recording-symbol {
  display: grid;
  place-items: center;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  color: #294f43;
  background: #e8eee4;
}
.recording-symbol::before {
  content: '';
  border-block: 6px solid transparent;
  border-left: 9px solid currentColor;
  margin-left: 2px;
}
.playing .recording-symbol {
  background: #294f43;
  color: white;
}
.playing .recording-symbol::before {
  margin: 0;
  border: 0;
  width: 10px;
  height: 10px;
  background: currentColor;
}
@media (prefers-color-scheme: dark) {
  .recording-symbol {
    color: #c4efd1;
    background: #304339;
  }
  .playing .recording-symbol {
    background: #a5d1b8;
    color: #18221f;
  }
}
</style>
