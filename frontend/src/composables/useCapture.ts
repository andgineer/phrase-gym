import { onMounted, onUnmounted, ref } from 'vue'
import { captureOwner, holdCapture, releaseCapture } from '../captureOwners'
import { retainMicrophone, setAudioSession } from '../audioSession'
import type { StoredTake } from '../practiceTypes'

type TakeContext = Pick<
  StoredTake,
  'target_id' | 'revision' | 'attempt_id' | 'exposure_id' | 'capture_event_id'
>
type CaptureOptions = {
  context: () => Promise<TakeContext>
  retain: (take: StoredTake) => Promise<void>
  finished: (take: StoredTake) => Promise<void>
  failure: () => Promise<void>
}
export function useCapture(options: CaptureOptions) {
  const capturing = ref(false)
  const requesting = ref(false)
  const processing = ref(false)
  const error = ref('')
  let stream: MediaStream | null = null
  let relinquishMicrophone: (() => void) | null = null
  let recorder: MediaRecorder | null = null
  let generation = 0
  let interrupted = false
  let stopping: Promise<void> | null = null
  let resolveStop: (() => void) | null = null

  function discardStream() {
    stream?.getTracks().forEach((track) => track.stop())
    relinquishMicrophone?.()
    relinquishMicrophone = null
    stream = null
  }
  async function stop(incomplete = false) {
    interrupted ||= incomplete
    if (recorder?.state === 'recording') recorder.stop()
    if (stopping) await stopping
  }
  async function release() {
    generation++
    requesting.value = false
    if (capturing.value) void options.failure().catch(() => {})
    await stop(true)
    discardStream()
    setAudioSession('auto')
  }
  async function start() {
    if (capturing.value || requesting.value || processing.value) return
    const token = ++generation
    requesting.value = true
    error.value = ''
    let leaseId: string | null = null
    try {
      if (!navigator.mediaDevices?.getUserMedia || !window.MediaRecorder)
        throw new Error('Recording is unavailable in this browser')
      if (!navigator.locks)
        throw new Error('Recording is unavailable in this browser. You can still use self-check.')
      const context = await options.context()
      if (token !== generation) return
      discardStream()
      setAudioSession('auto')
      // Native defaults avoid iOS capture failures when audio processing is forced off.
      const acquired = await navigator.mediaDevices.getUserMedia({ audio: true })
      if (token !== generation) {
        acquired.getTracks().forEach((track) => track.stop())
        return
      }
      stream = acquired
      relinquishMicrophone = retainMicrophone(acquired)
      const formats = ['audio/webm;codecs=opus', 'audio/mp4', 'audio/ogg;codecs=opus']
      const format = formats.find((type) => MediaRecorder.isTypeSupported(type))
      const capture = new MediaRecorder(stream, format ? { mimeType: format } : undefined)
      recorder = capture
      const settings = stream.getAudioTracks()[0]?.getSettings() ?? {}
      const allowed = [
        'sampleRate',
        'channelCount',
        'echoCancellation',
        'noiseSuppression',
        'autoGainControl',
      ]
      const capture_settings = Object.fromEntries(
        Object.entries(settings).filter(([key]) => allowed.includes(key)),
      )
      const take: StoredTake = {
        ...context,
        id: crypto.randomUUID(),
        captured_at: new Date().toISOString(),
        mime_type: capture.mimeType || format || '',
        duration_ms: null,
        complete: false,
        capture_settings,
        audio: new ArrayBuffer(0),
        owner: captureOwner,
        status: 'capturing',
      }
      await options.retain(take)
      if (token !== generation) return
      await holdCapture(take.id)
      leaseId = take.id
      if (token !== generation) {
        releaseCapture(take.id)
        return
      }
      const chunks: Blob[] = []
      let byteLength = 0
      let writes = Promise.resolve()
      const started = performance.now()
      interrupted = false
      stopping = new Promise((resolve) => {
        resolveStop = resolve
      })
      capture.ondataavailable = (event) => {
        if (event.data.size) {
          take.mime_type = event.data.type || capture.mimeType || take.mime_type
          chunks.push(event.data)
          byteLength += event.data.size
          // Leave headroom for the final encoder chunk below the server's 12 MiB limit.
          if (byteLength >= 8 * 1024 * 1024) {
            error.value = 'Recording reached the storage limit; the captured audio is retained.'
            void stop(true)
            void options.failure().catch(() => {})
          }
        }
        writes = writes
          .then(async () => {
            take.audio = await new Blob(chunks, { type: take.mime_type }).arrayBuffer()
            await options.retain({ ...take })
          })
          .catch((cause) => {
            error.value = String(cause)
          })
      }
      capture.onerror = () => {
        error.value = 'Recording was interrupted'
        void stop(true)
        void options.failure()
      }
      capture.onstop = async () => {
        capturing.value = false
        processing.value = true
        // The final data event precedes stop; release the input before routing playback.
        discardStream()
        setAudioSession('auto')
        try {
          await writes
          take.audio = await new Blob(chunks, { type: take.mime_type }).arrayBuffer()
          take.duration_ms = Math.round(performance.now() - started)
          take.complete = !interrupted && take.audio.byteLength > 0
          take.status = 'pending'
          await options.retain(take)
        } catch (cause) {
          error.value = String(cause)
        } finally {
          if (take.audio.byteLength)
            await options.finished(take).catch((cause) => {
              error.value = String(cause)
            })
          else {
            error.value = 'No audio was captured. Please record again.'
            discardStream()
            setAudioSession('auto')
            await options.failure().catch(() => {})
          }
          processing.value = false
          recorder = null
          resolveStop?.()
          stopping = null
        }
      }
      stream.getAudioTracks().forEach((track) => {
        track.onended = () => {
          void stop(true)
          void options.failure()
        }
      })
      capture.start(1000)
      capturing.value = true
    } catch (cause) {
      if (!capturing.value) {
        if (leaseId) releaseCapture(leaseId)
        recorder = null
        resolveStop?.()
        stopping = null
        discardStream()
        setAudioSession('auto')
      }
      error.value = String(cause)
      await options.failure().catch(() => {})
    } finally {
      if (token === generation) requesting.value = false
    }
  }
  function background() {
    if (document.hidden) void release()
  }
  onMounted(() => document.addEventListener('visibilitychange', background))
  onUnmounted(() => {
    void release()
    document.removeEventListener('visibilitychange', background)
  })
  return { capturing, requesting, processing, error, start, stop, release }
}
