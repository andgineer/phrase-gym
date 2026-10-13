<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouter } from 'vue-router'
import type { Language } from '../api'
import type { PracticeMessages } from '../practiceMessages'
import type { usePracticeSession } from '../composables/usePracticeSession'
import { languageNames } from '../i18n'
const props = defineProps<{ engine: ReturnType<typeof usePracticeSession>; pt: PracticeMessages }>()
const router = useRouter()
const requested = ref<{ language: Language; lessonId?: string; dialogueId?: string } | null>(null)
const { openSession: session, busy, error } = props.engine
const open = computed(
  () => session.value?.status === 'active' || session.value?.status === 'paused',
)
async function navigate() {
  const state = session.value ?? props.engine.session.value
  if (state) await router.push(`/learn/${state.manifest.language}/practice/${state.id}`)
}
async function resume() {
  if (session.value?.status === 'paused') await props.engine.transition('active', session.value)
  requested.value = null
  await navigate()
}
async function launch(language: Language, lessonId?: string, dialogueId?: string) {
  if (busy.value) return
  requested.value = { language, lessonId, dialogueId }
  await create()
}
async function create() {
  if (!requested.value) return
  const { language, lessonId, dialogueId } = requested.value
  await props.engine.start(language, lessonId, dialogueId, open.value)
  if (!error.value) {
    requested.value = null
    await navigate()
  }
}
defineExpose({ launch })
</script>
<template>
  <aside
    v-if="(open && engine.routeId.value !== session?.id) || requested || error"
    class="session-banner"
  >
    <template v-if="open && session && engine.routeId.value !== session.id">
      <span
        >{{ languageNames[session.manifest.language] }} · {{ session.cursor }} /
        {{ session.queue.length }}</span
      >
      <button :disabled="busy" @click="resume">{{ pt.resume }}</button>
    </template>
    <button v-if="requested && error" :disabled="busy" @click="create">
      {{ pt.retry }}
    </button>
    <p v-if="requested && busy" role="status">{{ pt.syncing }}</p>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
  </aside>
</template>
