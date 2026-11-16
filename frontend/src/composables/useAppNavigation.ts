import { onMounted, onUnmounted, ref } from 'vue'

export type Page = 'entry' | 'library' | 'coach' | 'settings'
export type Screen = Page | 'card'

export function pageFromHash(hash: string): Page {
  return hash === '#library' || hash === '#coach' || hash === '#settings'
    ? (hash.slice(1) as Page)
    : 'entry'
}

export function useAppNavigation(navigate: (page: Page) => Promise<void>) {
  const screen = ref<Screen>(pageFromHash(window.location.hash))
  let pending = Promise.resolve()

  function show(next: Screen) {
    screen.value = next
    const hash = next === 'entry' || next === 'card' ? '' : `#${next}`
    window.history.replaceState(
      null,
      '',
      `${window.location.pathname}${window.location.search}${hash}`,
    )
  }

  function select(next: Page) {
    pending = pending.then(() => navigate(next))
    return pending
  }

  const hashChanged = () => void select(pageFromHash(window.location.hash))
  onMounted(() => window.addEventListener('hashchange', hashChanged))
  onUnmounted(() => window.removeEventListener('hashchange', hashChanged))
  return { screen, show, select }
}
