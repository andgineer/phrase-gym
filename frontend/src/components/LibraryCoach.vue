<script setup lang="ts">
import { watch } from 'vue'
import { useCoachSubmission } from '../composables/useCoachSubmission'

const props = defineProps<{ text: string; hasResults: boolean }>()
const { sending, confirmation, error, submit, reset } = useCoachSubmission()
// A completed request belongs to its search, even if the learner edits during submission.
let submitted = ''
async function send() {
  submitted = props.text
  await submit(submitted)
  if (submitted !== props.text) reset()
}
watch(() => props.text, reset)
</script>

<template>
  <div class="library-coach">
    <p v-if="confirmation" role="status" class="muted">{{ confirmation }}</p>
    <button
      v-else
      :class="hasResults ? 'quiet coach-search-link' : 'primary coach-search-button'"
      :disabled="sending"
      @click="send"
    >
      <template v-if="hasResults">Not it? Send “{{ text }}” to Coach</template>
      <template v-else>
        <span aria-hidden="true">＋ </span>Send “{{ text }}” to Coach
        <small>Coach prepares a card for Added with Coach.</small>
      </template>
    </button>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
  </div>
</template>
