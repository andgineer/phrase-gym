import { onMounted, onUnmounted, ref } from 'vue'
import { api, requireServer, serverUnavailable, connectionRetrying } from '../api'
import { useCapture } from './useCapture'
import { usePracticeAudio } from './usePracticeAudio'
import { takeTransfers } from '../takeTransfers'
import type { RecordingReceipt, StoredTake } from '../practiceTypes'

type Context = Pick<
  StoredTake,
  'target_id' | 'revision' | 'attempt_id' | 'exposure_id' | 'capture_event_id'
>
export function useCardAudio(
  context: () => Promise<Context>,
  referenceUrl: () => string,
  failure: () => Promise<void>,
  currentScope: () => string,
  completed: () => Promise<void> = async () => {},
) {
  const takes = takeTransfers
  const audio = usePracticeAudio()
  const latestUrl = ref('')
  let localUrl = ''
  let live = true
  let generation = 0
  let captureScope = ''
  let captureReference = ''
  function local(take: StoredTake) {
    if (localUrl) URL.revokeObjectURL(localUrl)
    localUrl = URL.createObjectURL(new Blob([take.audio], { type: take.mime_type }))
    latestUrl.value = localUrl
  }
  async function refresh(targetId: string) {
    const token = ++generation
    const pending = (await takes.localTakes())
      .filter((t) => t.target_id === targetId && t.audio.byteLength)
      .sort((a, b) => b.captured_at.localeCompare(a.captured_at))[0]
    if (pending && live && token === generation) local(pending)
    const rows = await api<RecordingReceipt[]>(`/phrases/${targetId}/recordings`)
    const latest = rows[rows.length - 1]
    if (
      live &&
      token === generation &&
      latest &&
      (!pending || latest.captured_at > pending.captured_at)
    )
      latestUrl.value = `/api/v1/recordings/${latest.id}`
  }
  const capture = useCapture({
    context: async () => {
      captureScope = currentScope()
      captureReference = referenceUrl()
      return context()
    },
    retain: takes.retain,
    failure,
    finished: async (take) => {
      const scope = captureScope
      const reference = captureReference
      if (take.complete && currentScope() === scope) {
        try {
          await completed()
        } catch {}
      }
      if (!live || currentScope() !== scope) {
        void takes.flush()
        return
      }
      generation++
      local(take)
      void takes.flush()
      if (
        live &&
        take.complete &&
        !document.hidden &&
        !serverUnavailable.value &&
        !connectionRetrying.value
      ) {
        void audio.play(localUrl).then((ok) => {
          if (ok && live && currentScope() === scope && reference) void audio.play(reference)
        })
      }
    },
  })
  async function control() {
    if (capture.requesting.value) {
      await capture.release()
      await failure().catch(() => {})
    } else if (capture.capturing.value) await capture.stop()
    else if (audio.playing.value) audio.stop()
    else {
      requireServer()
      audio.stop()
      await capture.start()
    }
  }
  async function release() {
    generation++
    audio.stop()
    await capture.release()
    audio.stop()
  }
  function stopOwnedMedia() {
    if (capture.requesting.value || capture.capturing.value) void control()
    audio.stop()
  }
  onMounted(() => window.addEventListener('phrase-gym:stop-media', stopOwnedMedia))
  onUnmounted(() => {
    window.removeEventListener('phrase-gym:stop-media', stopOwnedMedia)
    live = false
    if (localUrl) URL.revokeObjectURL(localUrl)
  })
  return { takes, audio, capture, latestUrl, refresh, control, release }
}
