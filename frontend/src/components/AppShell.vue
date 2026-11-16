<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'
import type { Page, Screen } from '../composables/useAppNavigation'

defineProps<{ screen: Screen; initialized: boolean }>()
const emit = defineEmits<{ navigate: [page: Page] }>()
const pages: { page: Page; label: string }[] = [
  { page: 'entry', label: 'Practice' },
  { page: 'library', label: 'Library' },
  { page: 'coach', label: 'Coach' },
  { page: 'settings', label: 'Settings' },
]
const version = ref('')
onMounted(async () => {
  try {
    version.value = (await api<components['schemas']['Health']>('/health')).version
  } catch {
    // Practice remains usable if the version read fails.
  }
})
</script>

<template>
  <div class="shell">
    <div class="brand-row">
      <a class="brand" href="/" @click.prevent="emit('navigate', 'entry')">
        Phrase Gym<span>◌</span>
      </a>
      <span v-if="version" class="muted app-version">v{{ version }}</span>
    </div>
    <nav v-if="initialized" class="page-tabs" aria-label="Pages">
      <button
        v-for="item in pages"
        :key="item.page"
        :aria-current="(screen === 'card' ? 'entry' : screen) === item.page ? 'page' : undefined"
        @click="emit('navigate', item.page)"
      >
        {{ item.label }}
      </button>
    </nav>
    <main>
      <h1 class="visually-hidden">
        {{ pages.find((item) => item.page === screen)?.label ?? 'Practice' }}
      </h1>
      <slot />
    </main>
  </div>
</template>
