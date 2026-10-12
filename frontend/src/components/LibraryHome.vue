<script setup lang="ts">
import type { RouteLocationRaw } from 'vue-router'
import type { Language, Lesson, Phrase } from '../api'
import { languageNames } from '../i18n'
import type { Messages } from '../i18n'
defineProps<{
  t: Messages
  language: Language
  lessonQuery: string
  phraseQuery: string
  lessons: Lesson[]
  lessonTotal: number
  results: Phrase[]
  total: number
  selectedLesson: string
  hasCollection: boolean
  loading: boolean
  loadingMore: boolean
  loadingMoreLessons: boolean
  link: (kind: 'phrases' | 'lessons', id: string) => RouteLocationRaw
}>()
const emit = defineEmits<{
  query: ['q' | 'lessonQ', Event]
  more: []
  moreLessons: []
  'update:selectedLesson': [string]
}>()
</script>
<template>
  <section class="home">
    <p class="eyebrow">{{ languageNames[language] }}</p>
    <h1>{{ t.home }}</h1>
    <div class="card session-entry">
      <label
        >{{ t.lessonScope
        }}<input
          type="search"
          :placeholder="t.lessonFilter"
          :aria-label="t.lessonFilter"
          :value="lessonQuery"
          @input="emit('query', 'lessonQ', $event)"
      /></label>
      <select
        :aria-label="t.lessonScope"
        :value="selectedLesson"
        @change="emit('update:selectedLesson', ($event.target as HTMLSelectElement).value)"
      >
        <option value="">{{ t.allLessons }}</option>
        <option v-for="lesson in lessons" :key="lesson.id" :value="lesson.id">
          {{ lesson.title }}
        </option>
      </select>
      <button
        v-if="!loading && lessons.length < lessonTotal"
        :disabled="loadingMoreLessons"
        @click="emit('moreLessons')"
      >
        {{ t.more }}
      </button>
      <RouterLink v-if="selectedLesson" :to="link('lessons', selectedLesson)"
        >{{ t.selectLesson }} →</RouterLink
      >
      <button class="primary" disabled aria-describedby="session-note">{{ t.start }}</button>
      <p class="muted" id="session-note">{{ t.upcoming }}</p>
    </div>
    <section class="lookup">
      <label
        ><h2>{{ t.lookup }}</h2>
        <input
          type="search"
          :placeholder="t.search"
          :aria-label="t.lookup"
          :value="phraseQuery"
          @input="emit('query', 'q', $event)"
      /></label>
      <p v-if="!loading && !hasCollection" class="muted">{{ t.empty }}</p>
      <p v-else-if="!loading && !results.length">{{ t.noMatches }}</p>
      <ul class="phrase-list">
        <li v-for="item in results" :key="item.id">
          <RouterLink :to="link('phrases', item.id)" :lang="language"
            >{{ item.content.text }}<span>↗</span></RouterLink
          >
        </li>
      </ul>
      <button
        v-if="!loading && results.length < total"
        :disabled="loadingMore"
        @click="emit('more')"
      >
        {{ t.more }}
      </button>
    </section>
  </section>
</template>
