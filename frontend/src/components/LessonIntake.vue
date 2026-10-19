<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Language } from '../api'
import { onboardingMessages } from '../onboardingMessages'
import { useLessonIntake } from '../composables/useLessonIntake'
const props = defineProps<{ language: Language; locale: Language }>()
const messages = computed(() => onboardingMessages(props.locale))
const open = ref(false)
const {
  draft,
  status,
  busy,
  error,
  ready,
  title,
  phrase,
  context,
  note,
  locale: audioLocale,
  performer,
  file,
  submit,
  refresh,
  retryPreparation,
  restore,
  clear,
} = useLessonIntake(() => messages.value)
</script>
<template>
  <section class="card" :aria-label="messages.add">
    <button @click="open = !open" :aria-expanded="open">{{ messages.add }}</button>
    <div v-if="open" class="lesson-input">
      <h2 id="lesson-input-heading">{{ messages.add }}</h2>
      <p>{{ language === 'en' ? messages.supported : messages.unsupported }}</p>
      <form v-if="language === 'en' && !draft" @submit.prevent="submit">
        <fieldset :disabled="busy || !ready">
          <label>{{ messages.title }}<input v-model="title" required maxlength="300" /></label>
          <label
            >{{ messages.phrase }}<input v-model="phrase" required maxlength="200" lang="en"
          /></label>
          <label
            >{{ messages.context }}<textarea v-model="context" required maxlength="8000" />
          </label>
          <label>{{ messages.note }}<textarea v-model="note" maxlength="2000" /></label>
          <label>{{ messages.performer }}<input v-model="performer" required /></label>
          <label>{{ messages.locale }}<input v-model="audioLocale" required /></label>
          <label
            >{{ messages.audio
            }}<input
              type="file"
              accept=".wav,.mp3,.ogg,.m4a,.mp4"
              required
              @change="file = ($event.target as HTMLInputElement).files?.[0]"
          /></label>
          <button class="primary" type="submit">{{ messages.submit }}</button>
        </fieldset>
      </form>
      <template v-if="draft">
        <h3>{{ draft.submission.title }}</h3>
        <p lang="en">{{ draft.submission.phrases[0]?.text }}</p>
        <p>{{ draft.submission.context }}</p>
        <p role="status">
          {{
            status ? messages[status.status] : draft.receipt ? messages.accepted : messages.unsent
          }}
        </p>
        <p v-if="status?.reason" role="alert">{{ status.reason }}</p>
        <RouterLink
          v-if="status?.publication"
          :to="`/learn/en/lessons/${status.publication.lesson_id}`"
          >{{ messages.open }} →</RouterLink
        >
        <button v-if="!draft.receipt" :disabled="busy" @click="submit">{{ messages.retry }}</button>
        <button v-else :disabled="busy" @click="refresh">{{ messages.refresh }}</button>
        <button v-if="status?.status === 'failed'" :disabled="busy" @click="retryPreparation">
          {{ messages.retryPreparation }}
        </button>
        <button v-if="!draft.receipt" :disabled="busy" @click="clear">
          {{ messages.abandon }}
        </button>
        <button v-if="status?.status === 'published'" :disabled="busy" @click="clear">
          {{ messages.another }}
        </button>
      </template>
      <p v-if="busy" role="status">{{ messages.busy }}</p>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <button v-if="!ready" :disabled="busy" @click="restore">{{ messages.retryStorage }}</button>
      <button @click="open = false">{{ messages.close }}</button>
    </div>
  </section>
</template>
