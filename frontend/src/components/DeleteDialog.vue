<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { ApiError, api } from '../api'
import { fileCleanup } from '../cleanupStatus'
import type { DeletionResult, DeletionTarget } from '../libraryTypes'

const props = defineProps<{ target: DeletionTarget | null }>()
const emit = defineEmits<{ deleted: [result: DeletionResult]; cancel: [] }>()
const dialog = ref<HTMLDialogElement | null>(null)
const cancelButton = ref<HTMLButtonElement | null>(null)
const deleting = ref(false)
const error = ref('')
let completed = false

watch(
  () => props.target,
  async (target) => {
    error.value = ''
    completed = false
    if (!target) return dialog.value?.open && dialog.value.close()
    dialog.value?.showModal()
    await nextTick()
    cancelButton.value?.focus()
  },
)

async function confirm() {
  const target = props.target
  if (!target || deleting.value) return
  deleting.value = true
  error.value = ''
  try {
    const result = await api<DeletionResult>(`/phrases/${encodeURIComponent(target.id)}`, {
      method: 'DELETE',
    })
    fileCleanup.set(result.cleanup)
    finish(result)
  } catch (cause) {
    if (cause instanceof ApiError && cause.status === 404) {
      await fileCleanup.refresh()
      finish({ deleted_text: target.text, cleanup: fileCleanup.status.value })
    } else error.value = String(cause)
  } finally {
    deleting.value = false
  }
}

function finish(result: DeletionResult) {
  completed = true
  dialog.value?.close()
  emit('deleted', result)
}

function closed() {
  if (!completed) emit('cancel')
}
</script>

<template>
  <dialog
    ref="dialog"
    class="intake-dialog confirm-dialog"
    aria-labelledby="delete-heading"
    @cancel="deleting && $event.preventDefault()"
    @close="closed"
  >
    <template v-if="target">
      <h2 id="delete-heading">Delete “{{ target.text }}”?</h2>
      <p>
        The phrase, its reference audio and its saved recordings will be permanently removed. This
        cannot be undone.
      </p>
      <p v-if="error" role="alert" class="error">{{ error }}</p>
      <div class="actions">
        <button ref="cancelButton" :disabled="deleting" @click="dialog?.close()">Cancel</button>
        <button class="danger" :disabled="deleting" @click="confirm">Delete permanently</button>
      </div>
    </template>
  </dialog>
</template>
