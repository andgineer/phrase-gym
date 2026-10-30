<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api } from '../api'
import type { components } from '../api.generated'

type Settings = components['schemas']['Settings']
const settings = ref<Settings | null>(null)
const message = ref('')
onMounted(async () => {
  try {
    settings.value = await api<Settings>('/settings')
  } catch (cause) {
    message.value = String(cause)
  }
})
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
        <input v-model="settings.learning_timezone" aria-label="Learning timezone"
      /></label>
      <p>English cards and short English help are selected for this MVP.</p>
      <button @click="save">Save settings</button>
    </template>
    <p v-if="message" role="status">{{ message }}</p>
  </section>
</template>
