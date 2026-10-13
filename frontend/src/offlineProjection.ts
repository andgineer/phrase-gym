import type { AttemptEvidence, PracticeInput, PracticeSession } from './practiceTypes'

export async function localAttemptId(sessionId: string, step: number): Promise<string> {
  const namespace = Uint8Array.from(sessionId.replaceAll('-', '').match(/../g)!, (x) =>
    parseInt(x, 16),
  )
  const name = new TextEncoder().encode(`step:${step}`)
  const input = new Uint8Array(namespace.length + name.length)
  input.set(namespace)
  input.set(name, namespace.length)
  const bytes = new Uint8Array(await crypto.subtle.digest('SHA-1', input)).slice(0, 16)
  bytes[6] = (bytes[6]! & 15) | 80
  bytes[8] = (bytes[8]! & 63) | 128
  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`
}

export function projectEvent(
  state: PracticeSession,
  event: PracticeInput,
  attemptId: string,
): PracticeSession {
  const next: PracticeSession = JSON.parse(JSON.stringify(state))
  if (event.sequence <= next.last_sequence) return next
  if (event.step !== next.cursor) throw new Error('Pending action no longer addresses this card')
  next.last_sequence = event.sequence
  if (event.kind === 'dialogue_exposure')
    next.dialogue_exposures = [...(next.dialogue_exposures ?? []), event.occurred_at]
  if (event.kind === 'dialogue_begin') next.queue[0]!.introduced = true
  if (event.kind.startsWith('dialogue_')) return next
  const step = next.queue[event.step]!
  const item = next.manifest.targets[step.target_index]!
  const scriptSupport = (next.dialogue_exposures ?? []).some((time) => {
    const elapsed = Date.parse(event.occurred_at) - Date.parse(time)
    return elapsed >= 0 && elapsed <= 300000
  })
  let evidence = next.attempts.find((a) => a.step === event.step)
  if (!evidence) {
    if (event.kind !== 'present') throw new Error('Present the cue before responding')
    evidence = {
      id: attemptId,
      step: event.step,
      target_id: item.phrase.id,
      revision: item.phrase.revision,
      cue_id: item.cue_id,
      review_group_id: item.review_group_id,
      repair: step.repair,
      supported:
        scriptSupport ||
        next.attempts.some(
          (a) =>
            a.target_id === item.phrase.id &&
            a.revealed &&
            Date.parse(event.occurred_at) - Date.parse(a.finalized_at ?? a.presented_at) < 300000,
        ),
      original: null,
      revealed: false,
      expanded: false,
      technical_failure: false,
      capture_pending: false,
      rating: null,
      presented_at: event.occurred_at,
    } satisfies AttemptEvidence
    next.attempts.push(evidence)
  }
  if (!evidence.original) evidence.supported ||= scriptSupport
  if (['respond', 'answer', 'rate'].includes(event.kind) && !evidence.original)
    evidence.original = {
      modality: event.modality,
      supported: evidence.supported,
      occurred_at: event.occurred_at,
    }
  if (['answer', 'explain', 'reference'].includes(event.kind)) {
    evidence.supported = true
    evidence.revealed = true
  }
  if (event.kind === 'explain') evidence.expanded = true
  if (event.kind === 'respond') evidence.capture_pending = true
  if (event.kind === 'capture_complete') {
    evidence.revealed = true
    evidence.capture_pending = false
  }
  if (event.kind === 'failure') {
    evidence.technical_failure = true
    evidence.capture_pending = false
  }
  if (event.kind === 'detour') step.dialogue_turn = null
  if (event.kind === 'rate') {
    evidence.rating = event.rating
    evidence.finalized_at = event.occurred_at
    evidence.eligibility_reason = 'pending_receipt'
    next.cursor++
    next.version++
    if (next.cursor >= next.queue.length) next.status = 'completed'
  }
  next.updated_at = event.occurred_at
  return next
}
