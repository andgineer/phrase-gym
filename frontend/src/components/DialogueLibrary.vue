<script setup lang="ts">
import { ref, watch } from 'vue'
import { api } from '../api'
import type { Language } from '../api'
import type { Dialogue } from '../practiceTypes'
import type { PracticeMessages } from '../practiceMessages'
const props = defineProps<{ language: Language; pt: PracticeMessages }>()
const emit = defineEmits<{ start: [string] }>()
const dialogues = ref<Dialogue[]>([])
const error = ref('')
let generation = 0
async function load() {
  const token = ++generation
  dialogues.value = []
  error.value = ''
  try {
    const page = await api<{ items: Dialogue[] }>(`/dialogues?language=${props.language}&limit=100`)
    if (token === generation) dialogues.value = page.items
  } catch (cause) {
    if (token === generation) error.value = String(cause)
  }
}
watch(() => props.language, load, { immediate: true })
</script>
<template>
  <section v-if="dialogues.length || error" class="dialogue-library">
    <h2>{{ pt.dialogue }}</h2>
    <ul>
      <li v-for="dialogue in dialogues" :key="dialogue.id">
        <button @click="emit('start', dialogue.id)">{{ dialogue.content.title }} →</button>
      </li>
    </ul>
    <p v-if="error" role="alert" class="error">
      {{ error }} <button @click="load">{{ pt.retry }}</button>
    </p>
  </section>
</template>
