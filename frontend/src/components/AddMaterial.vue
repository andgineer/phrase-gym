<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
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
const state = ref<ImportStatus | null>(null)
const error = ref('')
const importing = ref(false)
let poll: number | null = null

async function pollImport(packageId: string) {
  try {
    state.value = await api<ImportStatus>(`/imports/${packageId}`)
    if (state.value.status === 'preparing' || state.value.status === 'pending') {
      poll = window.setTimeout(() => void pollImport(packageId), 1000)
    } else {
      importing.value = false
      if (state.value.status === 'added') emit('added')
    }
  } catch (cause) {
    error.value = String(cause)
    importing.value = false
  }
}

onMounted(async () => {
  try {
    const imports = await api<ImportStatus[]>('/imports')
    if (imports[0]) {
      state.value = imports[0]
      if (state.value.status === 'pending' || state.value.status === 'preparing') {
        await pollImport(state.value.package_id)
      }
    }
  } catch (cause) {
    error.value = String(cause)
  }
})

async function importPackage() {
  error.value = ''
  try {
    const value = JSON.parse(text.value) as unknown
    importing.value = true
    state.value = await api<ImportStatus>('/imports', {
      method: 'POST',
      body: JSON.stringify({ package: value }),
    })
    await pollImport(state.value.package_id)
  } catch (cause) {
    importing.value = false
    error.value = cause instanceof SyntaxError ? `Invalid JSON: ${cause.message}` : String(cause)
  }
}

async function retry() {
  if (!state.value) return
  importing.value = true
  error.value = ''
  state.value = await api<ImportStatus>(`/imports/${state.value.package_id}/retry`, {
    method: 'POST',
  })
  await pollImport(state.value.package_id)
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

onUnmounted(() => {
  if (poll !== null) window.clearTimeout(poll)
})
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
    <span v-if="state?.status === 'pending'" role="status">Pending Coach</span>
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
      <p v-if="state?.status === 'pending'" role="status">
        Pending Coach. You can keep studying while it is offline.
      </p>
      <p v-if="state?.status === 'preparing'" role="status">Preparing audio…</p>
      <button v-if="state?.status === 'preparing'" class="quiet" @click="retry">
        Retry interrupted preparation
      </button>
      <p v-if="state?.status === 'added'" role="status" class="success">
        Added {{ state.added_count }} phrases.
      </p>
      <div v-if="state?.status === 'error'" role="alert" class="error">
        <p>{{ state.error }}</p>
        <button @click="retry">Retry preparation</button>
      </div>
      <p v-if="error" role="alert" class="error pre">{{ error }}</p>
      <button class="quiet" @click="dialog?.close()">Close</button>
    </dialog>
  </section>
</template>
