import { computed, onMounted, ref } from 'vue'
import type { ExposureInput, Language, Phrase } from '../api'
import { flushExposures, retainExposure, sendExposure } from '../exposures'

export function useReferenceExposure() {
  const currentExposure = ref<ExposureInput | null>(null)
  const pendingExposure = ref<ExposureInput | null>(null)
  const sendingId = ref<string | null>(null)
  const exposureSaving = computed(() =>
    Boolean(pendingExposure.value && pendingExposure.value.id === sendingId.value),
  )
  function prepareExposure(item: Phrase, language: Language) {
    const payload: ExposureInput = {
      id: crypto.randomUUID(),
      target_id: item.id,
      revision: item.revision,
      language,
      kind: 'explanation',
    }
    retainExposure(payload)
    currentExposure.value = payload
    pendingExposure.value = payload
  }
  async function saveExposure() {
    const payload = pendingExposure.value
    if (!payload || sendingId.value === payload.id) return
    sendingId.value = payload.id
    try {
      if ((await sendExposure(payload)) && pendingExposure.value?.id === payload.id)
        pendingExposure.value = null
    } finally {
      if (sendingId.value === payload.id) sendingId.value = null
    }
  }
  onMounted(() => {
    flushExposures().catch(() => {})
  })
  return { currentExposure, pendingExposure, exposureSaving, prepareExposure, saveExposure }
}
