<script setup lang="ts">
import { useLibrary } from './composables/useLibrary'
import { languages, languageNames } from './i18n'
import SetupForm from './components/SetupForm.vue'
import PhraseMaterials from './components/PhraseMaterials.vue'
import LessonView from './components/LessonView.vue'
import LibraryHome from './components/LibraryHome.vue'

const {
  route,
  settings,
  native,
  useLearning,
  configuring,
  error,
  loading,
  phrase,
  lessons,
  lessonTotal,
  results,
  total,
  hasCollection,
  selectedLesson,
  playing,
  audioError,
  pendingExposure,
  exposureSaving,
  currentLanguage,
  t,
  firstLaunch,
  lessonQuery,
  phraseQuery,
  kind,
  selectedLanguage,
  explanation,
  lessonTitle,
  playReference,
  saveExposure,
  saveSettings,
  changeLanguage,
  goBack,
  link,
  updateQuery,
  loadMore,
  loadMoreLessons,
  loadingMore,
  loadingMoreLessons,
  loadRoute,
} = useLibrary()
</script>
<template>
  <div class="shell">
    <header>
      <RouterLink class="brand" :to="currentLanguage ? `/learn/${currentLanguage}` : '/'"
        >Phrase Gym<span>◌</span></RouterLink
      >
      <div class="header-controls" v-if="settings?.native_language">
        <label
          >{{ t.learning
          }}<select :aria-label="t.learning" :value="selectedLanguage" @change="changeLanguage">
            <option disabled value="">{{ t.choose }}</option>
            <option v-for="code in languages" :key="code" :value="code">
              {{ languageNames[code] }}
            </option>
          </select></label
        >
        <button class="quiet" @click="configuring = !configuring">{{ t.settings }}</button>
      </div>
    </header>
    <main>
      <p class="error" v-if="error" role="alert">
        {{ error }} <button @click="loadRoute">{{ t.retry }}</button>
      </p>
      <SetupForm
        v-if="firstLaunch || configuring"
        :t="t"
        :first-launch="firstLaunch"
        v-model:native="native"
        v-model:use-learning="useLearning"
        @save="saveSettings"
      />
      <p role="status" v-if="loading">{{ t.loading }}</p>
      <template
        v-if="
          settings?.native_language && !firstLaunch && !configuring && (!loading || kind === 'home')
        "
      >
        <section class="card" v-if="!currentLanguage">
          <h1>{{ t.choose }}</h1>
          <p v-if="route.path !== '/' && route.path !== '/learn'">{{ t.unavailable }}</p>
          <nav class="language-links">
            <RouterLink v-for="code in languages" :key="code" :to="`/learn/${code}`">{{
              languageNames[code]
            }}</RouterLink>
          </nav>
        </section>
        <template v-else>
          <button
            v-if="phrase || kind === 'lessons'"
            class="back quiet"
            :aria-label="t.back"
            @click="goBack"
          >
            ← {{ t.back }}
          </button>
          <PhraseMaterials
            v-if="phrase"
            :t="t"
            :phrase="phrase"
            :language="currentLanguage"
            :native-language="settings.native_language"
            :explanation="explanation"
            :playing="playing"
            :audio-error="audioError"
            :pending-exposure="pendingExposure"
            :exposure-saving="exposureSaving"
            :link="link"
            @reference="playReference"
            @retry-exposure="saveExposure"
          />
          <LessonView
            v-else-if="kind === 'lessons' && (!error || results.length > 0)"
            :t="t"
            :language="currentLanguage"
            :lesson-title="lessonTitle"
            :results="results"
            :total="total"
            :link="link"
            :loading-more="loadingMore"
            @more="loadMore"
          />
          <LibraryHome
            v-else-if="kind === 'home'"
            :t="t"
            :language="currentLanguage"
            :lesson-query="lessonQuery"
            :phrase-query="phraseQuery"
            :lessons="lessons"
            :lesson-total="lessonTotal"
            :results="results"
            :total="total"
            v-model:selected-lesson="selectedLesson"
            :has-collection="hasCollection"
            :loading="loading"
            :failed="Boolean(error)"
            :loading-more="loadingMore"
            :loading-more-lessons="loadingMoreLessons"
            :link="link"
            @query="updateQuery"
            @more="loadMore"
            @more-lessons="loadMoreLessons"
          />
        </template>
      </template>
    </main>
  </div>
</template>
