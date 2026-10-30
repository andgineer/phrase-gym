<script setup lang="ts">
import { onMounted } from 'vue'
import { fileCleanup } from '../cleanupStatus'

const { status, failure } = fileCleanup
onMounted(() => void fileCleanup.refresh())
</script>

<template>
  <div v-if="status.pending || failure" class="cleanup-notice">
    <p v-if="status.error" role="alert" class="error">{{ status.error }}</p>
    <p v-else-if="status.pending" role="status">
      Removing files of deleted phrases ({{ status.pending }} left).
    </p>
    <p v-if="failure" role="alert" class="error">{{ failure }}</p>
    <button v-if="status.pending" @click="fileCleanup.retry">Retry cleanup</button>
  </div>
</template>
