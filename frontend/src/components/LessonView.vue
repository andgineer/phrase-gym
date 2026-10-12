<script setup lang="ts">
import type { RouteLocationRaw } from 'vue-router'
import type { Language, Phrase } from '../api'
import type { Messages } from '../i18n'
defineProps<{
  t: Messages
  language: Language
  lessonTitle: string
  results: Phrase[]
  total: number
  loadingMore: boolean
  link: (kind: 'phrases' | 'lessons', id: string) => RouteLocationRaw
}>()
const emit = defineEmits<{ more: [] }>()
</script>
<template>
  <section class="card">
    <p class="eyebrow">{{ t.allPhrases }}</p>
    <h1 :lang="language">{{ lessonTitle }}</h1>
    <button class="primary" disabled aria-describedby="session-note">{{ t.lessonDrill }}</button>
    <p class="muted" id="session-note">{{ t.upcoming }}</p>
    <ul class="phrase-list">
      <li v-for="item in results" :key="item.id">
        <RouterLink :to="link('phrases', item.id)" :lang="language"
          >{{ item.content.text }} <span>↗</span></RouterLink
        >
      </li>
    </ul>
    <button v-if="results.length < total" :disabled="loadingMore" @click="emit('more')">
      {{ t.more }}
    </button>
  </section>
</template>
