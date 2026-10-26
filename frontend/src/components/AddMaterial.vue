<script setup lang="ts">
import { onUnmounted, ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'

type ImportStatus = components['schemas']['ImportStatus']
const emit = defineEmits<{ added: [] }>()
const text = ref('')
const state = ref<ImportStatus | null>(null)
const error = ref('')
const importing = ref(false)
let poll: number | null = null

async function pollImport(packageId: string) {
  try {
    state.value = await api<ImportStatus>(`/imports/${packageId}`)
    if (state.value.status === 'preparing') {
      poll = window.setTimeout(() => void pollImport(packageId), 100)
    } else {
      importing.value = false
      if (state.value.status === 'added') emit('added')
    }
  } catch (cause) {
    error.value = String(cause)
    importing.value = false
  }
}

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

onUnmounted(() => {
  if (poll !== null) window.clearTimeout(poll)
})
</script>

<template>
  <section class="panel intake">
    <details>
      <summary>Add material</summary>
      <p>Paste a Phrase Gym package or choose a UTF-8 JSON file.</p>
      <textarea v-model="text" aria-label="Phrase package JSON" rows="8" />
      <div class="actions">
        <label class="file"
          >Choose JSON file<input type="file" accept="application/json,.json" @change="chooseFile"
        /></label>
        <button :disabled="importing || !text.trim()" @click="importPackage">Import</button>
      </div>
      <p v-if="state?.status === 'preparing'" role="status">Preparing audio…</p>
      <p v-if="state?.status === 'added'" role="status" class="success">
        Added {{ state.added_count }} phrases.
      </p>
      <div v-if="state?.status === 'error'" role="alert" class="error">
        <p>{{ state.error }}</p>
        <button @click="retry">Retry preparation</button>
      </div>
      <p v-if="error" role="alert" class="error pre">{{ error }}</p>
    </details>
  </section>
</template>
