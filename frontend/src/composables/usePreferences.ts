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
  const savingSettings = ref(false)
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
  async function save() {
    if (!native.value || savingSettings.value) return false
    savingSettings.value = true
    try {
      settings.value = await api<Settings>('/settings', {
        method: 'PUT',
        body: JSON.stringify({
          native_language: native.value,
          ui_language_mode: useLearning.value ? 'learning' : 'native',
        }),
      })
      configuring.value = false
      return true
    } finally {
      savingSettings.value = false
    }
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
    savingSettings,
  }
}
