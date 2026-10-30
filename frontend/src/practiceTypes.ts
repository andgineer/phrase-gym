import type { components } from './api.generated'

type Upload = components['schemas']['RecordingUpload']
export type TakeSnapshot = Pick<
  Upload,
  'phrase_id' | 'view_token' | 'mode' | 'kind' | 'cue' | 'hint'
>
export type StoredTake = TakeSnapshot & {
  id: string
  captured_at: string
  mime_type: string
  duration_ms: number | null
  complete: boolean
  capture_settings: Record<string, string | number | boolean>
  audio: ArrayBuffer
  owner?: string
  status: 'capturing' | 'pending'
}
