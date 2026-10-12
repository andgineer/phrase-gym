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
  const locale = computed(() =>
    settings.value?.ui_language_mode === 'learning' && currentLanguage.value
      ? currentLanguage.value
      : (settings.value?.native_language ?? 'en'),
  )
  const t = computed(() => messages(locale.value))
  const firstLaunch = computed(() => settings.value !== null && !settings.value.native_language)
  async function load() {
    if (!settings.value) {
      settings.value = await api<Settings>('/settings')
      native.value = settings.value.native_language ?? ''
      useLearning.value = settings.value.ui_language_mode === 'learning'
    }
  }
  async function save() {
    if (!native.value) return
    settings.value = await api<Settings>('/settings', {
      method: 'PUT',
      body: JSON.stringify({
        native_language: native.value,
        ui_language_mode: useLearning.value ? 'learning' : 'native',
        last_learning_language:
          currentLanguage.value ?? settings.value?.last_learning_language ?? null,
      }),
    })
    configuring.value = false
  }
  function rememberLanguage(lang: Language) {
    if (settings.value) settings.value.last_learning_language = lang
    api('/settings/language', {
      method: 'PATCH',
      body: JSON.stringify({ last_learning_language: lang }),
    }).catch(() => {})
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
