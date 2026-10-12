import { computed, ref } from 'vue'
import type { Ref } from 'vue'
import { api } from '../api'
import type { Language, Settings } from '../api'
import { messages } from '../i18n'

export function usePreferences(currentLanguage: Ref<Language | null>) {
  const settings = ref<Settings | null>(null)
  const native = ref<Language | ''>('')
  const useLearning = ref(false)
  const configuring = ref(false)
  let initialization: Promise<void> | null = null
  let writes: Promise<unknown> = Promise.resolve()
  let pendingLanguage: Language | null = null
  let writingLanguage = false
  let languageVersion = 0
  const locale = computed(() =>
    settings.value?.ui_language_mode === 'learning' && currentLanguage.value
      ? currentLanguage.value
      : (settings.value?.native_language ?? 'en'),
  )
  const t = computed(() => messages(locale.value))
  const firstLaunch = computed(() => settings.value !== null && !settings.value.native_language)
  async function load() {
    if (settings.value) return
    if (!initialization) {
      initialization = api<Settings>('/settings')
        .then((value) => {
          settings.value = value
          native.value = value.native_language ?? ''
          useLearning.value = value.ui_language_mode === 'learning'
        })
        .finally(() => {
          initialization = null
        })
    }
    await initialization
  }
  function write<T>(operation: () => Promise<T>): Promise<T> {
    const result = writes.then(operation)
    writes = result.catch(() => {})
    return result
  }
  async function save() {
    if (!native.value) return
    const submittedLanguageVersion = languageVersion
    const body = JSON.stringify({
      native_language: native.value,
      ui_language_mode: useLearning.value ? 'learning' : 'native',
      last_learning_language:
        currentLanguage.value ?? settings.value?.last_learning_language ?? null,
    })
    const saved = await write(() => api<Settings>('/settings', { method: 'PUT', body }))
    // Full-save responses may lag a newer route's cached fallback.
    if (submittedLanguageVersion !== languageVersion && settings.value) {
      saved.last_learning_language = settings.value.last_learning_language
    }
    settings.value = saved
    configuring.value = false
  }
  async function drainLanguage() {
    if (writingLanguage) return
    writingLanguage = true
    try {
      while (pendingLanguage) {
        const lang = pendingLanguage
        pendingLanguage = null
        await write(() =>
          api('/settings/language', {
            method: 'PATCH',
            body: JSON.stringify({ last_learning_language: lang }),
          }),
        ).catch(() => {})
      }
    } finally {
      writingLanguage = false
    }
  }
  function rememberLanguage(lang: Language) {
    languageVersion += 1
    if (settings.value) settings.value.last_learning_language = lang
    pendingLanguage = lang
    void drainLanguage()
  }
  return {
    settings,
    native,
    useLearning,
    configuring,
    locale,
    t,
    firstLaunch,
    load,
    save,
    rememberLanguage,
  }
}
