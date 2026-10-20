import type { components } from '../tests/rust-contract/api.generated'
export type PracticeSession = components['schemas']['PracticeSession']
export type PracticeInput = components['schemas']['PracticeInput']
export type AttemptEvidence = components['schemas']['AttemptEvidence']
export type PlannedTarget = components['schemas']['PlannedTarget']
export type RecallRating = NonNullable<components['schemas']['PracticeInput']['rating']>
export type SessionStart = components['schemas']['SessionStart']
export type PracticeBatchReceipt = components['schemas']['PracticeBatchReceipt']
export type RecordingReceipt = components['schemas']['RecordingReceipt']
export type Dialogue = components['schemas']['Dialogue']
export type PendingEvent = { sessionId: string; event: PracticeInput }
export type StoredTake = {
  id: string
  target_id: string
  revision: number
  attempt_id?: string
  exposure_id?: string
  capture_event_id?: string
  captured_at: string
  mime_type: string
  duration_ms: number | null
  complete: boolean
  capture_settings: Record<string, string | number | boolean>
  audio: ArrayBuffer
  owner?: string
  status: 'capturing' | 'pending'
}
