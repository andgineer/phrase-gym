<script setup lang="ts">
import { ref } from 'vue'
import DueOverview from './DueOverview.vue'
import { onboardingMessages } from '../onboardingMessages'
import type { PracticeMessages } from '../practiceMessages'
import type { RouteLocationRaw } from 'vue-router'
import type { Language, Lesson, Phrase } from '../api'
import { languageNames } from '../i18n'
import type { Messages } from '../i18n'
defineProps<{
  publication: number
  t: Messages
  pt: PracticeMessages
  locale: Language
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
  failed: boolean
  loadingMore: boolean
  loadingMoreLessons: boolean
  link: (kind: 'phrases' | 'lessons', id: string) => RouteLocationRaw
}>()
const eligible = ref(false)
const emit = defineEmits<{
  published: []
  start: []
  practice: []
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
    <p v-if="!loading && !failed && !hasCollection">{{ onboardingMessages(locale).empty }}</p>
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
      <button
        class="primary"
        :disabled="loading || failed || !hasCollection || !eligible"
        @click="emit('start')"
      >
        {{ t.start }}
      </button>
    </div>
    <DueOverview
      :key="publication"
      :language="language"
      :lesson-id="selectedLesson"
      :pt="pt"
      :locale="locale"
      @eligible="eligible = $event"
      @practice="emit('practice')"
    />
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
      <p v-if="!loading && !failed && hasCollection && !results.length">{{ t.noMatches }}</p>
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
