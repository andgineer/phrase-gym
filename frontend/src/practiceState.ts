import { initialCard, reduceCard } from './reducer'
import type { CardState } from './reducer'
import type { PracticeInput, PracticeSession } from './practiceTypes'

export function sessionCard(session: PracticeSession, pending: PracticeInput[]): CardState {
  const evidence = session.attempts.find((a) => a.step === session.cursor)
  let state: CardState = evidence
    ? {
        revealed: evidence.revealed,
        expanded: evidence.expanded,
        exposedBeforeResponse: evidence.supported,
        originalResponse: evidence.original
          ? {
              modality: evidence.original.modality === 'recorded' ? 'recorded' : 'self_report',
              supported: evidence.original.supported,
            }
          : null,
      }
    : initialCard()
  for (const event of pending) {
    if (event.step !== session.cursor) continue
    if (event.kind === 'answer' || event.kind === 'explain' || event.kind === 'respond') {
      state = reduceCard(state, {
        type: event.kind,
        modality: event.modality === 'recorded' ? 'recorded' : 'self_report',
      })
      if (event.kind === 'respond') state = { ...state, revealed: true }
    }
    if (event.kind === 'reference')
      state = {
        ...state,
        revealed: true,
        exposedBeforeResponse: state.exposedBeforeResponse || state.originalResponse === null,
      }
  }
  return state
}
