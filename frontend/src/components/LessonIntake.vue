<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Language } from '../api'
import { onboardingMessages } from '../onboardingMessages'
import { useIntakeStatusPolling } from '../composables/useIntakeStatusPolling'
import { useLessonIntake } from '../composables/useLessonIntake'
const props = defineProps<{
  language: Language
  locale: Language
  beforeOpen?: () => Promise<void>
  practicing?: boolean
}>()
const messages = computed(() => onboardingMessages(props.locale))
const open = ref(false)
const {
  draft,
  status,
  busy,
  error,
  ready,
  phrase,
  note,
  submit,
  refresh,
  retryPreparation,
  restore,
  clear,
} = useLessonIntake(() => messages.value)
const emit = defineEmits<{ published: [] }>()
useIntakeStatusPolling(() => draft.value?.receipt?.generation_id, status, refresh)
watch(
  () => status.value?.publication?.publication_id,
  (id) => {
    if (id) emit('published')
  },
)
const interruptionReady = ref(true)
let opening = 0
let edits = 0
function dropText(event: DragEvent) {
  phrase.value = event.dataTransfer?.getData('text/plain') ?? phrase.value
  edits++
}
function close() {
  opening++
  open.value = false
}
async function toggle() {
  if (open.value) {
    close()
    return
  }
  const version = ++opening
  const previousEdits = edits
  // Invoke inside the original click gesture, before capture interruption can await.
  let clipboard: Promise<string>
  try {
    clipboard = navigator.clipboard?.readText().catch(() => '') ?? Promise.resolve('')
  } catch {
    clipboard = Promise.resolve('')
  }
  open.value = true
  interruptionReady.value = false
  const interruption = props.beforeOpen?.()
  void clipboard.then((text) => {
    if (open.value && opening === version && edits === previousEdits && !draft.value)
      phrase.value = text
  })
  try {
    await interruption
    if (opening === version) interruptionReady.value = true
  } catch (cause) {
    if (opening === version) error.value = String(cause)
  }
}
</script>
<template>
  <section class="phrase-drop" aria-label="Add phrase">
    <button @click="toggle" :aria-expanded="open" aria-label="Add phrase">+</button>
    <div
      v-if="open"
      class="lesson-input phrase-drop-panel"
      role="dialog"
      aria-modal="true"
      aria-labelledby="lesson-input-heading"
    >
      <h2 id="lesson-input-heading">Add phrase</h2>
      <p>
        {{
          language === 'en'
            ? 'Review one to five English items, one per line, then Send. Coach creates the practice material and reference speech.'
            : messages.unsupported
        }}
      </p>
      <form
        v-if="language === 'en' && !draft"
        @submit.prevent="submit"
        @dragover.prevent
        @drop.prevent="dropText"
      >
        <fieldset :disabled="busy || !ready || !interruptionReady">
          <label
            >{{ messages.phrase
            }}<textarea v-model="phrase" maxlength="4000" lang="en" @input="edits++" />
          </label>
          <label>{{ messages.note }}<textarea v-model="note" maxlength="2000" /></label>
          <button type="submit" :disabled="!phrase.trim()">Send</button>
        </fieldset>
      </form>
      <template v-if="draft">
        <!-- During a session the last batch's items could reveal answers still to be recalled. -->
        <template v-if="!practicing">
          <h3>{{ draft.submission.title }}</h3>
          <p v-for="item in draft.submission.phrases" :key="item.source_id" lang="en">
            {{ item.text }}
          </p>
          <p>{{ draft.submission.context }}</p>
        </template>
        <p role="status">
          {{
            status ? messages[status.status] : draft.receipt ? messages.accepted : messages.unsent
          }}
        </p>
        <p v-if="status?.reason" role="alert">{{ status.reason }}</p>
        <RouterLink
          v-if="status?.publication"
          @click="close"
          :to="`/learn/en/lessons/${status.publication.lesson_id}`"
          >{{ messages.open }} →</RouterLink
        >
        <button v-if="!draft.receipt" :disabled="busy" @click="submit">{{ messages.retry }}</button>
        <button v-else :disabled="busy" @click="refresh">{{ messages.refresh }}</button>
        <button v-if="status?.status === 'failed'" :disabled="busy" @click="retryPreparation">
          {{ messages.retryPreparation }}
        </button>
        <button
          v-if="
            !draft.receipt ||
            (status && ['failed', 'cancelled', 'superseded'].includes(status.status))
          "
          :disabled="busy"
          @click="clear"
        >
          {{ messages.abandon }}
        </button>
        <button v-if="status?.status === 'published'" :disabled="busy" @click="clear">
          {{ messages.another }}
        </button>
      </template>
      <p v-if="busy" role="status">{{ messages.busy }}</p>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
      <button v-if="!ready" :disabled="busy" @click="restore">{{ messages.retryStorage }}</button>
      <button @click="close">{{ messages.close }}</button>
    </div>
  </section>
</template>

<style scoped>
.phrase-drop-panel {
  position: fixed;
  inset: 5vh max(1rem, calc((100vw - 38rem) / 2));
  z-index: 20;
  overflow: auto;
  padding: 1.5rem;
  background: var(--card, white);
  color: var(--text, #222);
  border: 1px solid #888;
  border-radius: 1rem;
  box-shadow: 0 0 0 100vmax #0008;
}
.phrase-drop-panel textarea {
  width: 100%;
  min-height: 5rem;
}
</style>
