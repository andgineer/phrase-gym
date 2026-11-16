<script setup lang="ts">
import { ref } from 'vue'
import { coachText, useCoachSubmission } from '../composables/useCoachSubmission'

const text = ref('')
const { sending, confirmation, error, submit } = useCoachSubmission()
async function send() {
  if (await submit(text.value)) text.value = ''
}
function enter(event: KeyboardEvent) {
  if (event.isComposing) return
  event.preventDefault()
  void send()
}
</script>

<template>
  <form class="card coach-page" aria-label="Coach" @submit.prevent="send">
    <label for="coach-text">Phrase or word</label>
    <textarea
      id="coach-text"
      v-model="text"
      rows="4"
      aria-label="Phrase or word for Coach"
      maxlength="1000"
      autocomplete="off"
      enterkeyhint="send"
      @keydown.enter="enter"
    />
    <p class="muted">Add the sentence you met it in after the phrase, if you have it.</p>
    <button class="primary" type="submit" :disabled="!coachText(text) || sending">Send</button>
    <p v-if="confirmation" role="status" class="muted">{{ confirmation }}</p>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
  </form>
  <p class="muted">Coach prepares a card later. It appears in Library under Added with Coach.</p>
</template>
