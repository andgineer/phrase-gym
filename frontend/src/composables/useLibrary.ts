import { computed, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { api, language } from '../api'
import type { Lesson, LessonPage, Phrase, PhrasePage } from '../api'
import { languageNames } from '../i18n'
import { rememberLanguage, restoredLanguage } from '../learningLanguage'
import { usePreferences } from './usePreferences'
import { usePagination } from './usePagination'
import { useReferenceAudio } from './useReferenceAudio'
import { useReferenceExposure } from './useReferenceExposure'

export function useLibrary() {
  const route = useRoute()
  const router = useRouter()
  const error = ref('')
  const loading = ref(true)
  const phrase = ref<Phrase | null>(null)
  const lessons = ref<Lesson[]>([])
  const lessonTotal = ref(0)
  const results = ref<Phrase[]>([])
  const total = ref(0)
  const hasCollection = ref(false)
  const selectedLesson = ref('')
  const { pendingExposure, exposureSaving, prepareExposure, saveExposure } = useReferenceExposure()
  let requestVersion = 0
  const routeParts = computed(() => route.path.split('/').filter(Boolean))
  const currentLanguage = computed(() =>
    routeParts.value[0] === 'learn' ? language(routeParts.value[1]) : null,
  )
  const preferences = usePreferences(currentLanguage)
  const { settings, native, useLearning, configuring, savingSettings, firstLaunch, locale, t } =
    preferences
  const { playing, audioError, stopAudio, playReference } = useReferenceAudio(
    () => phrase.value,
    () => t.value.audioFailed,
  )
  const lessonQuery = computed(() =>
    typeof route.query.lessonQ === 'string' ? route.query.lessonQ : '',
  )
  const phraseQuery = computed(() => (typeof route.query.q === 'string' ? route.query.q : ''))
  const kind = computed(() => routeParts.value[2] ?? 'home')
  const selectedLanguage = computed(() => currentLanguage.value ?? '')
  const explanation = computed(() => {
    const nativeLanguage = settings.value?.native_language
    return nativeLanguage ? phrase.value?.content.explanations?.[nativeLanguage] : null
  })
  const lessonTitle = ref('')

  async function saveSettings() {
    if (savingSettings.value) return
    error.value = ''
    try {
      if (await preferences.save()) await loadRoute()
    } catch (cause) {
      error.value = String(cause)
    }
  }
  function changeLanguage(event: Event) {
    const next = language((event.target as HTMLSelectElement).value)
    if (next) router.push(`/learn/${next}`)
  }
  function goBack() {
    if (window.history.state.back) router.back()
    else router.push({ path: `/learn/${currentLanguage.value}`, query: route.query })
  }
  function link(type: 'phrases' | 'lessons', id: string) {
    return { path: `/learn/${currentLanguage.value}/${type}/${id}`, query: route.query }
  }
  function updateQuery(name: 'q' | 'lessonQ', event: Event) {
    const value = (event.target as HTMLInputElement).value
    router.replace({ query: { ...route.query, [name]: value || undefined } })
  }
  const phrasePages = usePagination({
    items: results,
    total,
    version: () => requestVersion,
    error,
    fetchPage: () => {
      const lang = currentLanguage.value
      const path =
        kind.value === 'lessons'
          ? `/lessons/${routeParts.value[3]}/phrases?language=${lang}&offset=${results.value.length}`
          : `/phrases/search?language=${lang}&q=${encodeURIComponent(phraseQuery.value)}&offset=${results.value.length}`
      return api<PhrasePage>(path)
    },
  })
  const lessonPages = usePagination({
    items: lessons,
    total: lessonTotal,
    version: () => requestVersion,
    error,
    fetchPage: () =>
      api<LessonPage>(
        `/lessons?language=${currentLanguage.value}&q=${encodeURIComponent(lessonQuery.value)}&offset=${lessons.value.length}`,
      ),
  })
  async function loadRoute() {
    const version = ++requestVersion
    phrasePages.reset()
    lessonPages.reset()
    total.value = 0
    lessonTotal.value = 0
    stopAudio()
    error.value = ''
    phrase.value = null
    results.value = []
    lessons.value = []
    pendingExposure.value = null
    audioError.value = ''
    loading.value = true
    try {
      if (currentLanguage.value) rememberLanguage(currentLanguage.value)
      await preferences.load()
      if (version !== requestVersion || firstLaunch.value || configuring.value) return
      const lang = currentLanguage.value
      if (!lang) {
        const restored = restoredLanguage()
        if (
          (route.path === '/' || route.path === '/learn' || route.path === '/learn/') &&
          restored
        ) {
          await router.replace(`/learn/${restored}`)
        }
        return
      }
      const base = `language=${lang}`
      if (kind.value === 'phrases' && routeParts.value[3]) {
        const item = await api<Phrase>(`/phrases/${routeParts.value[3]}?${base}`)
        if (version !== requestVersion || firstLaunch.value || configuring.value) return
        prepareExposure(item, lang)
        phrase.value = item
        loading.value = false
        void saveExposure()
      } else if (kind.value === 'lessons' && routeParts.value[3]) {
        const page = await api<PhrasePage>(`/lessons/${routeParts.value[3]}/phrases?${base}`)
        if (version !== requestVersion) return
        results.value = page.items
        total.value = page.total
        lessonTitle.value =
          page.items[0]?.lessons.find((l) => l.id === routeParts.value[3])?.title ?? ''
      } else if (kind.value === 'home') {
        const [lessonPage, phrasePage, collection] = await Promise.all([
          api<LessonPage>(`/lessons?${base}&q=${encodeURIComponent(lessonQuery.value)}`),
          api<PhrasePage>(`/phrases/search?${base}&q=${encodeURIComponent(phraseQuery.value)}`),
          api<PhrasePage>(`/phrases/search?${base}&limit=1`),
        ])
        if (version !== requestVersion) return
        lessons.value = lessonPage.items
        lessonTotal.value = lessonPage.total
        results.value = phrasePage.items
        total.value = phrasePage.total
        hasCollection.value = collection.total > 0
      } else {
        error.value = t.value.unavailable
      }
    } catch (cause) {
      if (version === requestVersion) error.value = String(cause)
    } finally {
      if (version === requestVersion) loading.value = false
    }
  }
  watch([currentLanguage, lessonQuery], () => {
    selectedLesson.value = ''
  })
  watch(configuring, (value) => {
    if (value) stopAudio()
    else loadRoute()
  })
  watch(() => route.fullPath, loadRoute)
  watch([locale, currentLanguage], () => {
    document.documentElement.lang = locale.value
    document.title = currentLanguage.value
      ? `Phrase Gym: ${languageNames[currentLanguage.value]}`
      : 'Phrase Gym'
  })
  onMounted(() => {
    loadRoute()
  })

  return {
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
    playing,
    audioError,
    pendingExposure,
    exposureSaving,
    currentLanguage,
    locale,
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
    loadMore: phrasePages.loadMore,
    loadMoreLessons: lessonPages.loadMore,
    loadingMore: phrasePages.loading,
    loadingMoreLessons: lessonPages.loading,
    loadRoute,
  }
}
