<script setup lang="ts">
import { ref, watch } from 'vue'
import type { Language } from '../api'
import { journalEntries } from '../practiceJournal'
import type { SessionPack } from '../sessionPack'
import type { PracticeMessages } from '../practiceMessages'
const props = defineProps<{ language: Language; pt: PracticeMessages }>()
const packs = ref<SessionPack[]>([])
watch(
  () => props.language,
  async (language) => {
    const saved = await journalEntries<SessionPack>('pack:').catch(() => [])
    if (props.language === language)
      packs.value = saved
        .map((e) => e.value)
        .filter((p) => p.session.manifest.language === language)
  },
  { immediate: true },
)
</script>
<template>
  <section v-if="packs.length" class="card">
    <h2>{{ pt.downloadedSessions }}</h2>
    <ul>
      <li v-for="pack in packs" :key="pack.session.id">
        <RouterLink :to="`/learn/${pack.session.manifest.language}/practice/${pack.session.id}`"
          >{{ pt.resume }} · {{ pack.session.manifest.targets.length }}</RouterLink
        >
      </li>
    </ul>
  </section>
</template>
