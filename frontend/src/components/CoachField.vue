<script setup lang="ts">
import { onUnmounted, ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'

type CoachSubmission = components['schemas']['CoachSubmission']

const confirmations: Record<CoachSubmission['outcome'], string> = {
  saved: 'Saved for Coach.',
  waiting: 'Already waiting for Coach.',
  in_library: 'Already in your Library.',
}
const CONFIRMATION_MS = 4000
const text = ref('')
const sending = ref(false)
const confirmation = ref('')
const error = ref('')
let hideConfirmation: number | undefined

async function submit() {
  const typed = text.value.trim()
  if (!typed || sending.value) return
  sending.value = true
  error.value = ''
  confirmation.value = ''
  try {
    const saved = await api<CoachSubmission>('/coach/requests', {
      method: 'POST',
      body: JSON.stringify({ text: typed }),
    })
    text.value = ''
    confirmation.value = confirmations[saved.outcome]
    window.clearTimeout(hideConfirmation)
    hideConfirmation = window.setTimeout(() => (confirmation.value = ''), CONFIRMATION_MS)
  } catch (cause) {
    error.value = cause instanceof Error ? cause.message : String(cause)
  } finally {
    sending.value = false
  }
}

onUnmounted(() => window.clearTimeout(hideConfirmation))
</script>

<template>
  <form class="coach-field" aria-label="Coach" @submit.prevent="submit">
    <div class="coach-input">
      <input
        v-model="text"
        type="text"
        aria-label="Phrase or word for Coach"
        placeholder="Phrase or word…"
        maxlength="1000"
        autocomplete="off"
        enterkeyhint="send"
      />
      <button type="submit" :disabled="!text.trim() || sending">Coach</button>
    </div>
    <p v-if="confirmation" role="status" class="coach-message muted">{{ confirmation }}</p>
    <p v-if="error" role="alert" class="coach-message error">{{ error }}</p>
  </form>
</template>
