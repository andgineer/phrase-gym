<script setup lang="ts">
import { onMounted, ref } from 'vue'
import SavedHistoryPanel from './SavedHistoryPanel.vue'

defineProps<{ phraseId: string; disabled: boolean; play: (url: string) => Promise<void> }>()
defineEmits<{ close: [] }>()
const dialog = ref<HTMLDialogElement | null>(null)
onMounted(() => dialog.value?.showModal())
</script>

<template>
  <dialog
    ref="dialog"
    class="intake-dialog recording-history-dialog"
    aria-labelledby="recording-history-heading"
    @close="$emit('close')"
  >
    <header>
      <h2 id="recording-history-heading">Recording history</h2>
      <button autofocus aria-label="Close recording history" @click="dialog?.close()">Close</button>
    </header>
    <SavedHistoryPanel :phrase-id="phraseId" :disabled="disabled" :play="play" />
  </dialog>
</template>
