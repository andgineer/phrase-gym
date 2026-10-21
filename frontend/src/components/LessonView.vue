<script setup lang="ts">
import { ref } from 'vue'
import DueOverview from './DueOverview.vue'
import type { PracticeMessages } from '../practiceMessages'
import type { RouteLocationRaw } from 'vue-router'
import type { Language, Phrase } from '../api'
import type { Messages } from '../i18n'
defineProps<{
  t: Messages
  pt: PracticeMessages
  locale: Language
  lessonId: string
  language: Language
  lessonTitle: string
  results: Phrase[]
  total: number
  loadingMore: boolean
  link: (kind: 'phrases' | 'lessons', id: string) => RouteLocationRaw
}>()
const eligible = ref(false)
const emit = defineEmits<{ more: []; start: []; practice: [] }>()
</script>
<template>
  <section class="card">
    <p class="eyebrow">{{ t.allPhrases }}</p>
    <h1 :lang="language">{{ lessonTitle }}</h1>
    <DueOverview
      :language="language"
      :lesson-id="lessonId"
      :pt="pt"
      :locale="locale"
      @eligible="eligible = $event"
      @practice="emit('practice')"
    />
    <button class="primary" :disabled="!eligible" @click="emit('start')">
      {{ t.lessonDrill }}
    </button>
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
