<script setup lang="ts">
import { computed, ref } from 'vue'
import { api } from '../api'
import type { HistoryItem } from '../studyTypes'

const props = defineProps<{
  items: HistoryItem[]
  disabled: boolean
  play: (url: string) => Promise<void>
  compare: (recording: HistoryItem['recordings'][number], reference: string) => Promise<void>
}>()
const emit = defineEmits<{ practise: [cardId: string]; changed: [] }>()
const opened = ref<string | null>(null)
const query = ref('')
const filtered = computed(() =>
  props.items.filter((item) =>
    `${item.source_label} ${item.item_id}`.toLowerCase().includes(query.value.toLowerCase()),
  ),
)
const error = ref('')

async function open(cardId: string) {
  try {
    await api(`/cards/${cardId}/expose`, { method: 'POST' })
    opened.value = cardId
  } catch (cause) {
    error.value = String(cause)
  }
}

async function setSuspended(item: HistoryItem) {
  try {
    await api(`/cards/${item.id}/${item.suspended ? 'restore' : 'suspend'}`, { method: 'POST' })
    emit('changed')
  } catch (cause) {
    error.value = String(cause)
  }
}
</script>

<template>
  <section v-if="props.items.length" class="card history">
    <h2>Library</h2>
    <p class="muted">Open a card for its wording, saved takes, and ungraded practice.</p>
    <input
      type="search"
      v-model="query"
      aria-label="Find card by source or item ID"
      placeholder="Find card by source or item ID"
    />
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <div v-for="item in filtered" :key="item.id" class="history-card">
      <button :disabled="disabled" @click="open(item.id)">
        {{ item.source_label }} · {{ item.item_id }} {{ item.suspended ? '(suspended)' : '' }}
      </button>
      <template v-if="opened === item.id">
        <h3>{{ item.answer }}</h3>
        <p v-if="item.due_at">Next scheduled return: {{ item.due_at.slice(0, 10) }}</p>
        <button :disabled="disabled" @click="emit('practise', item.id)">Practise freely</button>
        <button :disabled="disabled" @click="setSuspended(item)">
          {{ item.suspended ? 'Restore' : 'Suspend' }}
        </button>
        <button :disabled="disabled" @click="props.play(item.reference_url)">Play reference</button>
        <div v-for="(recording, takeIndex) in item.recordings" :key="recording.id" class="take">
          <span>Take {{ takeIndex + 1 }}</span>
          <button :disabled="disabled" @click="props.play(recording.url)">Replay saved take</button>
          <button :disabled="disabled" @click="props.compare(recording, item.reference_url)">
            Compare saved take
          </button>
        </div>
        <ul v-if="item.reviews.length">
          <li v-for="review in item.reviews" :key="review.reviewed_at">
            {{ review.reviewed_at.slice(0, 10) }} · {{ review.rating.replace('_', ' ') }} ·
            {{ review.response_mode }} · due {{ review.due_at.slice(0, 10) }}
          </li>
        </ul>
      </template>
    </div>
  </section>
</template>
