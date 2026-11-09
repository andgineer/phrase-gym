<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'
import { returnSpan } from '../returnSpan'

type Settings = components['schemas']['Settings']
type PaceChoice = components['schemas']['ReturnPaceChoice']
const paceNames: Record<Settings['return_pace'], string> = {
  often: 'Often',
  balanced: 'Balanced',
  rarely: 'Rarely',
}
const settings = ref<Settings | null>(null)
const paces = ref<PaceChoice[]>([])
const message = ref('')
onMounted(async () => {
  try {
    ;[settings.value, paces.value] = await Promise.all([
      api<Settings>('/settings'),
      api<PaceChoice[]>('/settings/return-paces'),
    ])
  } catch (cause) {
    message.value = String(cause)
  }
})
const gaps = (choice: PaceChoice) =>
  `${choice.good_gaps_days.map(returnSpan).join(', ')} · forget 1 in ${choice.forget_one_in}`
async function save() {
  if (!settings.value) return
  try {
    settings.value = await api<Settings>('/settings', {
      method: 'PUT',
      body: JSON.stringify(settings.value),
    })
    message.value = 'Settings saved. Existing phrases keep their prepared language.'
  } catch (cause) {
    message.value = String(cause)
  }
}
</script>

<template>
  <section class="card history">
    <h2>Settings</h2>
    <template v-if="settings">
      <label
        >Native language
        <select v-model="settings.native_language">
          <option value="ru">Russian</option>
          <option value="en">English</option>
        </select></label
      >
      <label
        >Learning timezone
        <input v-model="settings.learning_timezone" type="text" aria-label="Learning timezone"
      /></label>
      <label
        >Due phrases per day
        <input v-model.number="settings.daily_due_limit" type="number" min="0" inputmode="numeric"
      /></label>
      <label
        >New phrases per day
        <input v-model.number="settings.daily_new_limit" type="number" min="0" inputmode="numeric"
      /></label>
      <fieldset class="return-paces">
        <legend>How often phrases come back</legend>
        <label v-for="choice in paces" :key="choice.pace" class="return-pace">
          <input v-model="settings.return_pace" type="radio" :value="choice.pace" />
          <span
            >{{ paceNames[choice.pace] }}
            <small class="muted" :data-testid="`pace-${choice.pace}`">{{
              gaps(choice)
            }}</small></span
          >
        </label>
        <p class="muted">Gaps when every answer is Good; applies from each phrase's next grade.</p>
      </fieldset>
      <p>English cards and short English help are selected for this MVP.</p>
      <button @click="save">Save settings</button>
    </template>
    <p v-if="message" role="status">{{ message }}</p>
  </section>
</template>
