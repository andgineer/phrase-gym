import type { PracticeSession } from './practiceTypes'
export type CardState = {
  revealed: boolean
  expanded: boolean
  exposedBeforeResponse: boolean
  originalResponse: null | { modality: 'self_report' | 'recorded'; supported: boolean }
}
export function sessionCard(session: PracticeSession): CardState {
  const evidence = session.attempts.find((a) => a.step === session.cursor)
  return {
    revealed: evidence?.revealed ?? false,
    expanded: evidence?.expanded ?? false,
    exposedBeforeResponse: evidence?.supported ?? false,
    originalResponse: evidence?.original
      ? {
          modality: evidence.original.modality === 'recorded' ? 'recorded' : 'self_report',
          supported: evidence.original.supported,
        }
      : null,
  }
}
