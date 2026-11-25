import { onMounted, onUnmounted, ref } from 'vue'
import { isLearningLanguage, type LearningLanguage } from '../languages'

export type Page = 'entry' | 'library' | 'coach' | 'settings'
export type Screen = Page | 'card'
export type Address = { language: LearningLanguage; page: Page }

const pages: Page[] = ['library', 'coach', 'settings']
const isPage = (value: string): value is Page => (pages as string[]).includes(value)

// `#de/library`; an address without a language, including an old bookmark, opens English.
export function addressFromHash(hash: string): Address {
  const [first = '', second] = hash.replace(/^#/, '').split('/')
  if (isPage(first) && second === undefined) return { language: 'en', page: first }
  if (!isLearningLanguage(first)) return { language: 'en', page: 'entry' }
  return { language: first, page: second && isPage(second) ? second : 'entry' }
}

export function hashFor(language: LearningLanguage, screen: Screen): string {
  return screen === 'entry' || screen === 'card' ? `#${language}` : `#${language}/${screen}`
}

export function useAppNavigation(
  navigate: (page: Page, language: LearningLanguage) => Promise<void>,
) {
  const initial = addressFromHash(window.location.hash)
  const screen = ref<Screen>(initial.page)
  const language = ref<LearningLanguage>(initial.language)
  let pending = Promise.resolve()

  // Replacing the address keeps tabs and language buttons out of the browser history.
  function show(next: Screen) {
    screen.value = next
    window.history.replaceState(
      null,
      '',
      `${window.location.pathname}${window.location.search}${hashFor(language.value, next)}`,
    )
  }

  function select(next: Page, nextLanguage: LearningLanguage = language.value) {
    pending = pending.then(() => navigate(next, nextLanguage))
    return pending
  }

  const hashChanged = () => {
    const address = addressFromHash(window.location.hash)
    void select(address.page, address.language)
  }
  onMounted(() => window.addEventListener('hashchange', hashChanged))
  onUnmounted(() => window.removeEventListener('hashchange', hashChanged))
  return { screen, language, show, select }
}
