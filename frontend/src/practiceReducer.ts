import type { Card, CompletionResult, Outcome, PracticeState, Progress } from './studyTypes'

export type PracticeView = {
  card: Card | null
  progress: Progress | null
  message: string
  detailsOpen: boolean
}
export type PendingCompletion = {
  completion_id: string
  view_token: string
  phrase_id: string
  outcome: Outcome
}

export const initialView = (): PracticeView => ({
  card: null,
  progress: null,
  message: '',
  detailsOpen: false,
})

export function openedState(state: PracticeState): PracticeView {
  return {
    card: state.card,
    progress: state.progress,
    message: state.message ?? '',
    detailsOpen: false,
  }
}

// A late response for a superseded appearance must not replace the current card.
export function updatedCard(view: PracticeView, card: Card, detailsOpen?: boolean): PracticeView {
  if (view.card?.view_token !== card.view_token) return view
  return { ...view, card, detailsOpen: detailsOpen ?? view.detailsOpen }
}

export function pendingFor(view: PracticeView, outcome: Outcome, id: string): PendingCompletion {
  if (!view.card) throw new Error('No practice card is open')
  return {
    completion_id: id,
    view_token: view.card.view_token,
    phrase_id: view.card.phrase_id,
    outcome,
  }
}

export type CompletionStep = { view: PracticeView; reconcile: boolean }

// A replayed or out-of-date acknowledgement is reconciled with the server's current view.
export function completedPractice(
  view: PracticeView,
  pending: PendingCompletion,
  result: CompletionResult,
): CompletionStep {
  if (
    result.replayed ||
    !result.state ||
    result.receipt.completion_id !== pending.completion_id ||
    view.card?.view_token !== pending.view_token
  )
    return { view, reconcile: true }
  return { view: openedState(result.state), reconcile: false }
}

export function progressLine(card: Card, progress: Progress | null): string {
  const mode = { daily: 'Daily', extra: 'Extra', practice: 'Practice' }[card.mode]
  if (!progress || card.mode === 'practice') return mode
  return card.kind === 'recall'
    ? `${mode} · ${progress.due_completed + 1} / ${progress.due_allowance} due`
    : `${mode} · ${progress.new_completed + 1} / ${progress.new_allowance} new`
}
