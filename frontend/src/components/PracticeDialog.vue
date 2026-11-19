<script setup lang="ts">
import { onMounted, ref, useId } from 'vue'

defineProps<{ title: string }>()
defineEmits<{ close: [] }>()
const headingId = useId()
const dialog = ref<HTMLDialogElement | null>(null)
onMounted(() => dialog.value?.showModal())
</script>

<template>
  <dialog
    ref="dialog"
    class="intake-dialog practice-dialog"
    :aria-labelledby="headingId"
    @close="$emit('close')"
  >
    <header>
      <h2 :id="headingId">{{ title }}</h2>
      <button autofocus :aria-label="`Close ${title.toLowerCase()}`" @click="dialog?.close()">
        Close
      </button>
    </header>
    <slot />
  </dialog>
</template>
