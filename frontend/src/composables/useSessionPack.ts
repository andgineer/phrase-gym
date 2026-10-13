import { ref, watch } from 'vue'
import type { Ref } from 'vue'
import { downloadPack, verifyPack } from '../sessionPack'
import type { PracticeSession } from '../practiceTypes'

export function useSessionPack(session: Ref<PracticeSession | null>) {
  const downloaded = ref(false)
  const busy = ref(false)
  const progress = ref('')
  const error = ref('')
  let generation = 0
  watch(
    () => session.value?.id,
    async (id) => {
      const token = ++generation
      downloaded.value = false
      error.value = ''
      try {
        const pack = id ? await verifyPack(id) : null
        if (token === generation) downloaded.value = Boolean(pack)
      } catch (cause) {
        if (token === generation) error.value = String(cause)
      }
    },
    { immediate: true },
  )
  async function download() {
    const state = session.value
    if (!state || busy.value) return
    const token = generation
    busy.value = true
    downloaded.value = false
    error.value = ''
    try {
      await downloadPack(state, (done, total) => {
        progress.value = `${done} / ${total}`
      })
      if (token === generation) downloaded.value = true
    } catch (cause) {
      error.value = `Download incomplete: ${String(cause)}. Device storage may be full. Pending actions and audio are retained; free space outside this app and retry.`
    } finally {
      busy.value = false
    }
  }
  return { downloaded, busy, progress, error, download }
}
