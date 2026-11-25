<script setup lang="ts">
import { ref } from 'vue'
import CoachInbox from './CoachInbox.vue'
import { useCoachInbox } from '../composables/useCoachInbox'
import { coachText, useCoachSubmission } from '../composables/useCoachSubmission'
import type { LearningLanguage } from '../languages'

const props = defineProps<{ language: LearningLanguage }>()
const text = ref('')
const { sending, confirmation, error, submit } = useCoachSubmission(props.language)
const inbox = useCoachInbox()
async function send() {
  const outcome = await submit(text.value)
  if (!outcome) return
  text.value = ''
  // A repeated rejection or a resubmitted failure changes what the inbox shows.
  if (outcome === 'rejected' || outcome === 'saved') void inbox.load()
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
  <CoachInbox
    :items="inbox.items.value"
    :busy="inbox.busy.value"
    :error="inbox.error.value"
    @dismiss="inbox.dismiss"
    @retry="inbox.retry"
  />
</template>
