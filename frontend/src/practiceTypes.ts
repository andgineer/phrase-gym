export type StoredTake = {
  id: string
  phrase_id: string
  attempt_id: string
  captured_at: string
  mime_type: string
  duration_ms: number | null
  complete: boolean
  capture_settings: Record<string, string | number | boolean>
  audio: ArrayBuffer
  owner?: string
  status: 'capturing' | 'pending'
}
