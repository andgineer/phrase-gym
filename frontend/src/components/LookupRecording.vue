<script setup lang="ts">
import { computed, watch } from 'vue'
import type { ExposureInput, Phrase } from '../api'
import type { PracticeMessages } from '../practiceMessages'
import { useCardAudio } from '../composables/useCardAudio'
const props = defineProps<{
  phrase: Phrase
  exposure: ExposureInput | null
  pt: PracticeMessages
  saveExposure: () => Promise<void>
}>()
const media = useCardAudio(
  async () => {
    const target = props.phrase
    await props.saveExposure()
    if (
      !props.exposure ||
      props.exposure.target_id !== target.id ||
      props.exposure.revision !== target.revision
    )
      throw new Error('The reference exposure must be retained before recording')
    return {
      target_id: target.id,
      revision: target.revision,
      exposure_id: props.exposure.id,
    }
  },
  () =>
    props.phrase.content.reference_audio_id
      ? `/api/v1/media/${props.phrase.content.reference_audio_id}`
      : '',
  async () => {},
  () => `${props.phrase.id}:${props.phrase.revision}`,
)
const { capturing, requesting, processing, error } = media.capture
const { playing, error: audioError } = media.audio
const { error: takeError, pendingTakes } = media.takes
const { latestUrl } = media
const disabled = computed(
  () =>
    !(capturing.value || requesting.value || playing.value) &&
    (!props.exposure || processing.value),
)
watch(
  () => `${props.phrase.id}:${props.phrase.revision}`,
  async () => {
    await media.release()
    latestUrl.value = ''
    try {
      await media.refresh(props.phrase.id)
    } catch {}
  },
  { immediate: true },
)
</script>
<template>
  <div class="lookup-recording">
    <div class="audio-controls">
      <button
        class="microphone"
        :disabled="disabled"
        :aria-label="
          requesting
            ? pt.cancelPermission
            : capturing
              ? pt.stopRecording
              : playing
                ? pt.stop
                : pt.record
        "
        @click="media.control"
      >
        {{ capturing || playing || requesting ? '■' : '🎙' }}
      </button>
      <button
        v-if="phrase.audio_status === 'ready'"
        :aria-label="pt.reference"
        :disabled="capturing || requesting"
        @click="media.audio.play(`/api/v1/media/${phrase.content.reference_audio_id}`)"
      >
        ▶ {{ pt.reference }}
      </button>
      <button
        :disabled="!latestUrl || capturing || requesting"
        @click="media.audio.play(latestUrl)"
      >
        ▶ {{ pt.mine }}
      </button>
    </div>
    <p v-if="capturing">{{ pt.stopHint }}</p>
    <p v-if="pendingTakes">{{ pt.pending }} ({{ pendingTakes }})</p>
    <p v-if="error || audioError || takeError" role="alert" class="error">
      {{ error || audioError || takeError }}
      <button
        @click="audioError ? media.audio.retry() : error ? media.control() : media.takes.flush()"
      >
        {{ pt.retry }}
      </button>
    </p>
  </div>
</template>
