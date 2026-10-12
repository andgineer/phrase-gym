import { onUnmounted, ref } from 'vue'
import { api } from '../api'
import { journalEntries } from '../practiceJournal'
import { useCapture } from './useCapture'
import { usePracticeAudio } from './usePracticeAudio'
import { useTakeJournal } from './useTakeJournal'
import type { RecordingReceipt, StoredTake } from '../practiceTypes'

type Context = Pick<
  StoredTake,
  'target_id' | 'revision' | 'attempt_id' | 'exposure_id' | 'capture_event_id'
>
export function useCardAudio(
  context: () => Promise<Context>,
  referenceUrl: () => string,
  failure: () => Promise<void>,
  completed: () => Promise<void> = async () => {},
) {
  const takes = useTakeJournal()
  const audio = usePracticeAudio()
  const latestUrl = ref('')
  let localUrl = ''
  let live = true
  let generation = 0
  function local(take: StoredTake) {
    if (localUrl) URL.revokeObjectURL(localUrl)
    localUrl = URL.createObjectURL(new Blob([take.audio], { type: take.mime_type }))
    latestUrl.value = localUrl
  }
  async function refresh(targetId: string) {
    const token = ++generation
    const pending = (await journalEntries<StoredTake>('take:'))
      .map((e) => e.value)
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
    context,
    retain: takes.retain,
    failure,
    finished: async (take) => {
      if (take.complete) {
        try {
          await completed()
        } catch {}
      }
      if (!live) {
        void takes.flush()
        return
      }
      generation++
      local(take)
      void takes.flush()
      if (live && take.complete && !document.hidden) {
        void audio.play(localUrl).then((ok) => {
          if (ok && live && referenceUrl()) void audio.play(referenceUrl())
        })
      }
    },
  })
  async function control() {
    if (capture.capturing.value) await capture.stop()
    else if (audio.playing.value) audio.stop()
    else {
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
  onUnmounted(() => {
    live = false
    if (localUrl) URL.revokeObjectURL(localUrl)
  })
  return { takes, audio, capture, latestUrl, refresh, control, release }
}
