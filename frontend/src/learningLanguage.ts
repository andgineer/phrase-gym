import { language } from './api'
import type { Language } from './api'

const key = 'phrase-gym.last-learning-language.v1'

export function restoredLanguage(): Language | null {
  try {
    return language(localStorage.getItem(key) ?? undefined)
  } catch {
    return null
  }
}

export function rememberLanguage(value: Language): void {
  try {
    localStorage.setItem(key, value)
  } catch {
    // This convenience must not prevent access when browser storage is unavailable.
  }
}
