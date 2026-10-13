import { computed, ref, watch } from 'vue'
import type { Ref } from 'vue'
import { downloadPack, verifyPack, packReady } from '../sessionPack'
import type { PracticeSession } from '../practiceTypes'

export function useSessionPack(session: Ref<PracticeSession | null>) {
  const downloaded = computed(() => Boolean(session.value && packReady.value[session.value.id]))
  const busy = ref(false)
  const progress = ref('')
  const error = ref('')
  let generation = 0
  watch(
    () => session.value?.id,
    async (id) => {
      const token = ++generation
      error.value = ''
      try {
        if (id) await verifyPack(id)
      } catch (cause) {
        if (token === generation) error.value = String(cause)
      }
    },
    { immediate: true },
  )
  async function download() {
    const state = session.value
    if (!state || busy.value) return
    busy.value = true
    error.value = ''
    try {
      await downloadPack(state, (done, total) => {
        progress.value = `${done} / ${total}`
      })
    } catch (cause) {
      error.value = `Download incomplete: ${String(cause)}. Device storage may be full. Pending actions and audio are retained; free space outside this app and retry.`
    } finally {
      busy.value = false
    }
  }
  return { downloaded, busy, progress, error, download }
}
