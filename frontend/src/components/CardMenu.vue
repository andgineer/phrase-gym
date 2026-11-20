<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
defineProps<{ disabled: boolean; showHistory: boolean }>()
const emit = defineEmits<{ remove: []; history: [] }>()
const open = ref(false)
const root = ref<HTMLElement | null>(null)
const toggle = ref<HTMLButtonElement | null>(null)
const focusToggle = () => toggle.value?.focus()
defineExpose({ focusToggle })
function choose(action: 'history' | 'remove') {
  open.value = false
  focusToggle()
  if (action === 'history') emit('history')
  else emit('remove')
}
function outside(event: PointerEvent) {
  if (!root.value?.contains(event.target as Node)) open.value = false
}
function key(event: KeyboardEvent) {
  if (event.key === 'Escape' && open.value) {
    open.value = false
    focusToggle()
  }
}
onMounted(() => {
  document.addEventListener('pointerdown', outside)
  document.addEventListener('keydown', key)
})
onUnmounted(() => {
  document.removeEventListener('pointerdown', outside)
  document.removeEventListener('keydown', key)
})
</script>
<template>
  <div ref="root" class="card-menu">
    <button
      ref="toggle"
      aria-label="More actions"
      :aria-expanded="open"
      :disabled="disabled"
      @click="open = !open"
    >
      ⋯
    </button>
    <div v-if="open" class="card-menu-panel">
      <button v-if="showHistory" @click="choose('history')">Recording history</button>
      <button class="danger" @click="choose('remove')">Delete phrase…</button>
    </div>
  </div>
</template>
