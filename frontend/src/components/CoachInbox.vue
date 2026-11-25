<script setup lang="ts">
import type { CoachOutcome } from '../composables/useCoachInbox'
import { languageName } from '../languages'

defineProps<{ items: CoachOutcome[]; busy: string | null; error: string }>()
const emit = defineEmits<{ dismiss: [requestId: string]; retry: [requestId: string] }>()
</script>

<template>
  <section v-if="items.length || error" class="card coach-inbox" aria-label="Coach results">
    <ul v-if="items.length" class="coach-results" aria-label="Coach results">
      <li v-for="item in items" :key="item.request_id" :data-outcome="item.outcome">
        <p v-if="item.outcome === 'corrected' && item.correction">
          <s :lang="item.learning_language">{{ item.correction.original }}</s> →
          <strong :lang="item.learning_language">{{ item.correction.corrected }}</strong>
        </p>
        <p v-else>
          <strong :lang="item.learning_language">“{{ item.text }}”</strong>
        </p>
        <p class="muted">
          {{ languageName(item.learning_language) }} ·
          <template v-if="item.outcome === 'corrected'">Corrected and added with Coach</template>
          <template v-else-if="item.outcome === 'rejected'">Not added</template>
          <template v-else>Coach could not finish</template>
        </p>
        <p v-if="item.reason" class="coach-reason">{{ item.reason }}</p>
        <div class="coach-result-actions">
          <button
            v-if="item.outcome === 'failed'"
            :disabled="busy !== null"
            @click="emit('retry', item.request_id)"
          >
            Retry
          </button>
          <button
            class="quiet"
            :aria-label="`Dismiss “${item.text}”`"
            :disabled="busy !== null"
            @click="emit('dismiss', item.request_id)"
          >
            Dismiss
          </button>
        </div>
      </li>
    </ul>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
  </section>
</template>
