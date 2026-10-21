<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import { useTakeJournal } from './composables/useTakeJournal'
import { usePracticeSession } from './composables/usePracticeSession'
import { practiceMessages } from './practiceMessages'
import type { Language } from './api'
import { flushExposures } from './exposures'
import PracticePanel from './components/PracticePanel.vue'
import SessionLauncher from './components/SessionLauncher.vue'
import ConnectionNotice from './components/ConnectionNotice.vue'
import { useServerRetry } from './composables/useServerRetry'
import DialogueLibrary from './components/DialogueLibrary.vue'
import { useLibrary } from './composables/useLibrary'
import { languages, languageNames } from './i18n'
import SetupForm from './components/SetupForm.vue'
import PhraseMaterials from './components/PhraseMaterials.vue'
import LessonView from './components/LessonView.vue'
import LibraryHome from './components/LibraryHome.vue'
import LessonIntake from './components/LessonIntake.vue'

const {
  route,
  settings,
  native,
  useLearning,
  configuring,
  savingSettings,
  error,
  loading,
  phrase,
  lessons,
  lessonTotal,
  results,
  total,
  hasCollection,
  selectedLesson,
  currentExposure,
  pendingExposure,
  exposureSaving,
  locale,
  currentLanguage,
  t,
  firstLaunch,
  lessonQuery,
  phraseQuery,
  kind,
  selectedLanguage,
  explanation,
  lessonTitle,
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
const publication = ref(0)
const practicePanel = ref<InstanceType<typeof PracticePanel> | null>(null)
const practice = usePracticeSession()
const transfers = useTakeJournal()
async function retryTransfers() {
  try {
    await practice.flush()
  } catch {}
  await transfers.flush()
}
const {
  retry: retryConnection,
  connectionRetrying,
  serverUnavailable,
} = useServerRetry(async () => {
  await flushExposures()
  if (!(await practice.load())) throw new Error('Session retry failed')
  if (!(await loadRoute(true))) throw new Error('Library retry failed')
  await transfers.recover()
}, practice.load)
const router = useRouter()
const pt = computed(() => practiceMessages(locale.value))
const launcher = ref<InstanceType<typeof SessionLauncher> | null>(null)
function startSession(lessonId?: string, dialogueId?: string) {
  if (currentLanguage.value)
    void launcher.value?.launch(currentLanguage.value, lessonId, dialogueId)
}
function practiceNow(lessonId?: string) {
  if (currentLanguage.value)
    void launcher.value?.launch(currentLanguage.value, lessonId, undefined, true)
}
async function openIntake() {
  await practicePanel.value?.openIntake()
}
function published() {
  publication.value++
  void loadRoute(true)
}
function leavePractice() {
  void router.push(`/learn/${practice.session.value?.manifest.language ?? currentLanguage.value}`)
}
</script>
<template>
  <div class="shell">
    <header>
      <LessonIntake
        :language="currentLanguage ?? 'en'"
        :locale="locale"
        :before-open="openIntake"
        @published="published"
      />
      <RouterLink class="brand" :to="currentLanguage ? `/learn/${currentLanguage}` : '/'"
        >Phrase Gym<span>◌</span></RouterLink
      >
      <div class="header-controls" v-if="settings?.native_language">
        <label
          >{{ t.learning
          }}<select
            :aria-label="t.learning"
            :value="selectedLanguage"
            :disabled="serverUnavailable || connectionRetrying"
            @change="changeLanguage"
          >
            <option disabled value="">{{ t.choose }}</option>
            <option v-for="code in languages" :key="code" :value="code">
              {{ languageNames[code] }}
            </option>
          </select></label
        >
        <button
          class="quiet"
          :disabled="savingSettings || serverUnavailable || connectionRetrying"
          @click="configuring = !configuring"
        >
          {{ t.settings }}
        </button>
      </div>
    </header>
    <main>
      <ConnectionNotice
        v-if="serverUnavailable || connectionRetrying"
        :pt="pt"
        :busy="connectionRetrying"
        @retry="retryConnection"
      />
      <div v-if="transfers.pendingTakes.value && (kind !== 'practice' || configuring)">
        <p role="status">{{ pt.pending }} ({{ transfers.pendingTakes.value }})</p>
        <p v-if="transfers.error.value" role="alert" class="error">{{ transfers.error.value }}</p>
        <button @click="retryTransfers">{{ pt.retry }}</button>
      </div>
      <div v-show="!serverUnavailable && !connectionRetrying">
        <p class="error" v-if="error" role="alert">
          {{ error }} <button @click="loadRoute()">{{ t.retry }}</button>
        </p>
        <SetupForm
          v-if="firstLaunch || configuring"
          :t="t"
          :first-launch="firstLaunch"
          :saving="savingSettings"
          v-model:native="native"
          v-model:use-learning="useLearning"
          @save="saveSettings"
        />
        <p role="status" v-if="loading">{{ t.loading }}</p>
        <template
          v-if="
            settings?.native_language &&
            !firstLaunch &&
            !configuring &&
            (!loading || kind === 'home')
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
            <SessionLauncher ref="launcher" :engine="practice" :pt="pt" />
            <PracticePanel
              ref="practicePanel"
              :key="practice.session.value?.id"
              v-if="
                kind === 'practice' &&
                practice.session.value?.manifest.language === currentLanguage &&
                practice.session.value?.id === practice.routeId.value
              "
              :engine="practice"
              :pt="pt"
              :native-language="settings.native_language as Language"
              :locale="locale"
              @leave="leavePractice"
              @practice="practiceNow(practice.session.value?.manifest.lesson_id ?? undefined)"
            />
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
              :pt="pt"
              :current-exposure="currentExposure"
              :save-exposure="saveExposure"
              :t="t"
              :phrase="phrase"
              :language="currentLanguage"
              :native-language="settings.native_language"
              :explanation="explanation"
              :pending-exposure="pendingExposure"
              :exposure-saving="exposureSaving"
              :link="link"
              @retry-exposure="saveExposure"
            />
            <LessonView
              v-else-if="kind === 'lessons' && (!error || results.length > 0)"
              :t="t"
              :language="currentLanguage"
              :lesson-title="lessonTitle"
              :lesson-id="route.path.split('/')[4]!"
              :pt="pt"
              :locale="locale"
              :results="results"
              :total="total"
              :link="link"
              :loading-more="loadingMore"
              @more="loadMore"
              @start="startSession(route.path.split('/')[4])"
              @practice="practiceNow(route.path.split('/')[4])"
            />
            <LibraryHome
              :publication="publication"
              v-else-if="kind === 'home'"
              :t="t"
              :language="currentLanguage"
              :pt="pt"
              :locale="locale"
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
              @start="startSession(selectedLesson)"
              @practice="practiceNow(selectedLesson)"
              @more-lessons="loadMoreLessons"
              @published="loadRoute()"
            />
            <DialogueLibrary
              v-if="kind === 'home'"
              :language="currentLanguage"
              :pt="pt"
              @start="(id) => startSession(undefined, id)"
            />
          </template>
        </template>
      </div>
    </main>
  </div>
</template>
