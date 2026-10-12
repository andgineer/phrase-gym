<script setup lang="ts">
import type { Language } from '../api'
import { languages, languageNames } from '../i18n'
import type { Messages } from '../i18n'
defineProps<{ t: Messages; native: Language | ''; useLearning: boolean; firstLaunch: boolean }>()
const emit = defineEmits<{
  save: []
  'update:native': [Language]
  'update:useLearning': [boolean]
}>()
</script>
<template>
  <section class="card setup">
    <p class="eyebrow">Phrase Gym</p>
    <h1>{{ firstLaunch ? t.welcome : t.settings }}</h1>
    <p>{{ t.setup }}</p>
    <form @submit.prevent="emit('save')">
      <label
        >{{ t.native
        }}<select
          aria-label="Native language"
          :value="native"
          @change="emit('update:native', ($event.target as HTMLSelectElement).value as Language)"
        >
          <option disabled value="">{{ t.native }}</option>
          <option v-for="code in languages" :key="code" :value="code">
            {{ languageNames[code] }}
          </option>
        </select></label
      >
      <label class="check"
        ><input
          type="checkbox"
          :checked="useLearning"
          @change="emit('update:useLearning', ($event.target as HTMLInputElement).checked)"
        />{{ t.learningUI }}</label
      >
      <button class="primary" type="submit" :disabled="!native">{{ t.save }}</button>
    </form>
  </section>
</template>
