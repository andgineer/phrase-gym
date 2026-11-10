<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'

type Settings = components['schemas']['Settings']
type PaceChoice = components['schemas']['ReturnPaceChoice']
const paceNames: Record<Settings['return_pace'], string> = {
  often: 'Often',
  balanced: 'Balanced',
  rarely: 'Rarely',
}
const paceCosts: Partial<Record<Settings['return_pace'], string>> = {
  often: 'fewer slip',
  rarely: 'more slip',
}
const settings = ref<Settings | null>(null)
const targetChoices = [1, 2, 3, 4, 5, 6]
// Removing the target also lifts a pause, so a later choice starts active.
const weeklyTarget = computed({
  get: () => settings.value?.weekly_target ?? null,
  set: (target: number | null) => {
    if (!settings.value) return
    settings.value.weekly_target = target
    if (target === null) settings.value.weekly_target_paused = false
  },
})
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
const summary = (choice: PaceChoice) =>
  [
    `${choice.first_month_returns}× first month`,
    `${choice.first_year_returns}× first year`,
    paceCosts[choice.pace],
  ]
    .filter(Boolean)
    .join(' · ')
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
      <fieldset class="settings-group">
        <legend>How often a phrase you keep answering Good comes back</legend>
        <label v-for="choice in paces" :key="choice.pace" class="settings-option">
          <input v-model="settings.return_pace" type="radio" :value="choice.pace" />
          <span
            >{{ paceNames[choice.pace] }}
            <small class="muted" :data-testid="`pace-${choice.pace}`">{{
              summary(choice)
            }}</small></span
          >
        </label>
        <p class="muted">Again or Hard bring it back sooner; each button shows when.</p>
      </fieldset>
      <fieldset class="settings-group">
        <legend>Weekly target</legend>
        <label
          >Practice days per week
          <select v-model="weeklyTarget">
            <option :value="null">No target</option>
            <option v-for="days in targetChoices" :key="days" :value="days">{{ days }}</option>
          </select></label
        >
        <label v-if="settings.weekly_target !== null" class="settings-option">
          <input v-model="settings.weekly_target_paused" type="checkbox" />
          <span>Pause weekly target</span>
        </label>
        <p class="muted">Any rating or Skip counts the day. A missed day or week owes nothing.</p>
      </fieldset>
      <p>English cards and short English help are selected for this MVP.</p>
      <button @click="save">Save settings</button>
    </template>
    <p v-if="message" role="status">{{ message }}</p>
  </section>
</template>
