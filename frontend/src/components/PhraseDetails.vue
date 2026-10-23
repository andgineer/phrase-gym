<script setup lang="ts">
import { computed } from 'vue'
import type { Language, Phrase } from '../api'
import { messages } from '../i18n'
const props = defineProps<{ phrase: Phrase }>()
const details = computed(() => props.phrase.english?.details ?? null)
const task = computed(() => props.phrase.english?.task)
const labels = computed(() => messages((details.value?.language ?? 'en') as Language))
</script>
<template>
  <section v-if="details" class="explanation phrase-details" :lang="details.language">
    <template v-if="task?.kind === 'word_sense'">
      <p class="details-word" :lang="phrase.language">
        <strong>{{ task.word }}</strong> — {{ task.sense }}
      </p>
      <h2>{{ labels.examples }}</h2>
      <ul :lang="phrase.language">
        <li v-for="context in task.contexts" :key="context.situation_id">{{ context.answer }}</li>
      </ul>
    </template>
    <h2>{{ labels.translation }}</h2>
    <p class="details-translation">{{ details.translation }}</p>
    <h2>{{ labels.meaning }}</h2>
    <p>{{ details.meaning }}</p>
    <h2>{{ labels.usage }}</h2>
    <p>{{ details.usage }}</p>
    <template v-if="details.forms">
      <h2>{{ labels.forms }}</h2>
      <p>{{ details.forms }}</p>
    </template>
  </section>
</template>
