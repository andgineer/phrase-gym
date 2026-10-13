<script setup lang="ts">
import { useSessionPack } from '../composables/useSessionPack'
import type { usePracticeSession } from '../composables/usePracticeSession'
import type { PracticeMessages } from '../practiceMessages'
import { updateWaiting, applyUpdate } from '../pwa'
const props = defineProps<{
  engine: ReturnType<typeof usePracticeSession>
  pt: PracticeMessages
  captureBusy: boolean
  pendingTakes: number
}>()
const emit = defineEmits<{ retry: [] }>()
const pack = useSessionPack(props.engine.session)
const { downloaded, busy: downloading, progress, error } = pack
</script>
<template>
  <div class="download-controls">
    <button
      :disabled="downloading || captureBusy || engine.busy.value || engine.restoring.value"
      @click="pack.download"
    >
      {{ pt.download }}
    </button>
    <span v-if="downloading" role="status">{{ pt.downloading }} {{ progress }}</span>
    <span v-else-if="downloaded" role="status">{{ pt.downloaded }}</span>
    <p v-if="engine.offline.value || engine.pending.value.length" role="status">
      {{ pt.localPending }} <button @click="emit('retry')">{{ pt.retry }}</button>
    </p>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <button
      v-if="
        updateWaiting &&
        engine.session.value &&
        ['paused', 'completed', 'superseded'].includes(engine.session.value.status)
      "
      :disabled="
        Boolean(
          engine.pending.value.length || pendingTakes || captureBusy || engine.restoring.value,
        )
      "
      @click="applyUpdate"
    >
      {{ pt.update }}
    </button>
  </div>
</template>
