/** Later takes and help must preserve the original response evidence. */
export type CardState = {
  revealed: boolean
  expanded: boolean
  exposedBeforeResponse: boolean
  originalResponse: null | { modality: 'self_report' | 'recorded'; supported: boolean }
}
export type CardAction = {
  type: 'explain' | 'answer' | 'respond'
  modality?: 'self_report' | 'recorded'
}
export const initialCard = (): CardState => ({
  revealed: false,
  expanded: false,
  exposedBeforeResponse: false,
  originalResponse: null,
})

export function reduceCard(state: CardState, action: CardAction): CardState {
  if (action.type === 'explain')
    return {
      ...state,
      revealed: true,
      expanded: true,
      exposedBeforeResponse: state.exposedBeforeResponse || state.originalResponse === null,
    }
  if (action.type === 'respond' || action.type === 'answer') {
    const response = state.originalResponse ?? {
      modality: action.modality ?? 'self_report',
      supported: state.exposedBeforeResponse,
    }
    return {
      ...state,
      revealed: state.revealed || action.type === 'answer',
      originalResponse: response,
    }
  }
  return state
}

export function referenceCard(): CardState {
  return reduceCard(initialCard(), { type: 'explain' })
}
