<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
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
const statuses = ref<ImportStatus[]>([])
const pendingCount = computed(
  () =>
    statuses.value.filter((status) => status.status === 'pending' || status.status === 'preparing')
      .length,
)
const failedCount = computed(
  () => statuses.value.filter((status) => status.status === 'error').length,
)
const error = ref('')
const statusError = ref('')
const importing = ref(false)
let poll: number | null = null
let loaded = false
let mounted = false

function upsert(status: ImportStatus) {
  const previous = statuses.value.find((item) => item.package_id === status.package_id)
  statuses.value = [
    status,
    ...statuses.value.filter((item) => item.package_id !== status.package_id),
  ]
  if (status.status === 'added' && previous?.status !== 'added') emit('added')
}

async function refreshStatuses() {
  try {
    const next = await api<ImportStatus[]>('/imports')
    if (!mounted) return
    if (loaded) {
      const previous = new Map(statuses.value.map((status) => [status.package_id, status.status]))
      if (
        next.some(
          (status) => status.status === 'added' && previous.get(status.package_id) !== 'added',
        )
      )
        emit('added')
    }
    statuses.value = next
    loaded = true
    statusError.value = ''
  } catch (cause) {
    statusError.value = String(cause)
  } finally {
    if (mounted) poll = window.setTimeout(() => void refreshStatuses(), 1000)
  }
}

onMounted(() => {
  mounted = true
  void refreshStatuses()
})

async function importPackage() {
  error.value = ''
  importing.value = true
  try {
    const value = JSON.parse(text.value) as unknown
    const status = await api<ImportStatus>('/imports', {
      method: 'POST',
      body: JSON.stringify({ package: value }),
    })
    upsert(status)
  } catch (cause) {
    error.value = cause instanceof SyntaxError ? `Invalid JSON: ${cause.message}` : String(cause)
  } finally {
    importing.value = false
  }
}

async function retry(packageId: string) {
  error.value = ''
  try {
    const status = await api<ImportStatus>(`/imports/${packageId}/retry`, { method: 'POST' })
    upsert(status)
  } catch (cause) {
    error.value = String(cause)
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

onUnmounted(() => {
  mounted = false
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
    <span v-if="pendingCount" role="status">Pending Coach</span>
    <span v-if="failedCount" role="status">Import needs attention</span>
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
      <ul v-if="statuses.length" class="import-statuses" aria-label="Import requests">
        <li v-for="status in statuses" :key="status.package_id">
          <strong>{{ status.package_id }}</strong>
          <p v-if="status.status === 'pending'" role="status">
            Pending Coach. You can keep studying while it is offline.
          </p>
          <p v-if="status.status === 'preparing'" role="status">Preparing audio…</p>
          <button
            v-if="status.status === 'preparing'"
            class="quiet"
            @click="retry(status.package_id)"
          >
            Retry interrupted preparation
          </button>
          <p v-if="status.status === 'added'" role="status" class="success">
            Added {{ status.added_count }} phrases.
          </p>
          <div v-if="status.status === 'error'" role="alert" class="error">
            <p>{{ status.error }}</p>
            <button @click="retry(status.package_id)">Retry preparation</button>
          </div>
        </li>
      </ul>
      <p v-if="error" role="alert" class="error pre">{{ error }}</p>
      <p v-if="statusError" role="alert" class="error pre">{{ statusError }}</p>
      <button class="quiet" @click="dialog?.close()">Close</button>
    </dialog>
  </section>
</template>
