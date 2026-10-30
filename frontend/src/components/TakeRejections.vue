<script setup lang="ts">
import { ref } from 'vue'
import { takeTransfers } from '../takeTransfers'

const failure = ref('')

async function discard(id: string) {
  failure.value = ''
  try {
    await takeTransfers.discard(id)
  } catch (cause) {
    failure.value = String(cause)
  }
}
</script>

<template>
  <div v-for="take in takeTransfers.rejected.value" :key="take.id" role="alert" class="error">
    <p>A recording kept on this device cannot be saved: {{ take.message }}</p>
    <button @click="discard(take.id)">Discard recording</button>
  </div>
  <p v-if="failure" role="alert" class="error">{{ failure }}</p>
</template>
