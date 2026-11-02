<script setup lang="ts">
import { ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'

type ImportStatus = components['schemas']['ImportStatus']
const props = defineProps<{ beforeOpen: () => Promise<void> }>()
const emit = defineEmits<{ added: [] }>()
const text = ref('')
const open = ref(false)
const interruptionReady = ref(true)
const dialog = ref<HTMLDialogElement | null>(null)
const opener = ref<HTMLButtonElement | null>(null)
const saved = ref<ImportStatus | null>(null)
const error = ref('')
const importing = ref(false)

async function importPackage() {
  error.value = ''
  saved.value = null
  importing.value = true
  try {
    const value = JSON.parse(text.value) as unknown
    saved.value = await api<ImportStatus>('/imports', {
      method: 'POST',
      body: JSON.stringify({ package: value }),
    })
    if (saved.value.status === 'added') emit('added')
  } catch (cause) {
    error.value = cause instanceof SyntaxError ? `Invalid JSON: ${cause.message}` : String(cause)
  } finally {
    importing.value = false
  }
}

async function chooseFile(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (file) text.value = await file.text()
}

async function toggle() {
  if (open.value) return dialog.value?.close()
  interruptionReady.value = false
  dialog.value?.showModal()
  open.value = true
  try {
    await props.beforeOpen()
    interruptionReady.value = true
  } catch (cause) {
    error.value = String(cause)
  }
}

function closed() {
  open.value = false
  opener.value?.focus()
}

function trapFocus(event: KeyboardEvent) {
  if (event.key !== 'Tab' || !dialog.value) return
  const controls = [
    ...dialog.value.querySelectorAll<HTMLElement>('button, textarea, input'),
  ].filter((element) => !element.hasAttribute('disabled') && element.getClientRects().length > 0)
  if (!controls.length) return
  if (event.shiftKey && document.activeElement === controls[0]) {
    event.preventDefault()
    controls.at(-1)?.focus()
  } else if (!event.shiftKey && document.activeElement === controls.at(-1)) {
    event.preventDefault()
    controls[0]?.focus()
  }
}
</script>

<template>
  <section class="phrase-drop" aria-label="Add material">
    <button
      ref="opener"
      class="add-phrase"
      aria-label="Add material"
      :aria-expanded="open"
      @click="toggle"
    >
      +
    </button>
    <dialog
      ref="dialog"
      class="intake-dialog"
      aria-labelledby="intake-heading"
      @close="closed"
      @keydown="trapFocus"
    >
      <h2 id="intake-heading">Add material</h2>
      <p>Paste a Phrase Gym package or choose a UTF-8 JSON file.</p>
      <textarea v-model="text" aria-label="Phrase package JSON" rows="8" />
      <p v-if="!interruptionReady" role="status">Finishing current recording…</p>
      <div class="actions">
        <label class="file"
          >Choose JSON file<input type="file" accept="application/json,.json" @change="chooseFile"
        /></label>
        <button :disabled="importing || !text.trim() || !interruptionReady" @click="importPackage">
          Import
        </button>
      </div>
      <p v-if="saved?.status === 'added'" role="status" class="success">
        Added {{ saved.added_count }} phrases.
      </p>
      <p v-else-if="saved" role="status">Saved. The phrases appear after Coach prepares audio.</p>
      <p v-if="error" role="alert" class="error pre">{{ error }}</p>
      <button class="quiet" @click="dialog?.close()">Close</button>
    </dialog>
  </section>
</template>
