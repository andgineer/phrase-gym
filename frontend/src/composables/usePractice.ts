import { ref } from 'vue'
import { ApiError, api } from '../api'
import {
  completedPractice,
  extendedPractice,
  initialView,
  openedState,
  pendingFor,
  updatedCard,
  type PendingCompletion,
  type PendingExtra,
} from '../practiceReducer'
import type { Card, CompletionResult, ExtraResult, Outcome, PracticeState } from '../studyTypes'

const changed = 'This card changed in another tab or a new day began.'
const practiceClosed = 'This Practice card changed in another tab or a new day began.'

export function usePractice() {
  const view = ref(initialView())
  const busy = ref(false)
  const error = ref('')
  const pending = ref<PendingCompletion | null>(null)
  const pendingExtra = ref<PendingExtra | null>(null)
  let lessonId: string | null = null
  // Each open or close starts a new epoch; results from an earlier one never load a card.
  let epoch = 0

  function post<T>(path: string, body: object) {
    return api<T>(`/practice/${path}`, { method: 'POST', body: JSON.stringify(body) })
  }

  async function load(path: string, body: object): Promise<boolean> {
    const mine = ++epoch
    busy.value = true
    error.value = ''
    // Another open abandons a failed Extra press, so its Retry cannot apply much later.
    pendingExtra.value = null
    try {
      const state = await post<PracticeState>(path, body)
      if (mine !== epoch) return false
      view.value = openedState(state)
      pending.value = null
      return state.card !== null
    } catch (cause) {
      if (mine === epoch) error.value = String(cause)
      return false
    } finally {
      busy.value = false
    }
  }

  function openDaily(lesson: string | null = lessonId) {
    lessonId = lesson
    return load('daily', { lesson_id: lesson })
  }

  const openPhrase = (phraseId: string) => load('phrase', { phrase_id: phraseId })

  // Reconciliation only reads, so it cannot close another tab's view; Daily makes any change.
  async function reconcile(): Promise<boolean | null> {
    const mine = epoch
    busy.value = true
    try {
      const state = await api<PracticeState>('/practice/current')
      if (mine !== epoch) return null
      view.value = openedState(state)
      pending.value = null
      return state.card !== null
    } catch (cause) {
      if (mine === epoch) error.value = String(cause)
      return null
    } finally {
      busy.value = false
    }
  }

  // Practice never falls back to Daily; closing its card returns to the phrase's details.
  async function resync(conflict: boolean, acknowledged = ''): Promise<boolean> {
    if (view.value.card?.mode === 'practice') {
      view.value = initialView()
      pending.value = null
      error.value = conflict ? practiceClosed : ''
      return false
    }
    const found = await reconcile()
    if (found === null) return false
    if (conflict) error.value = found ? `${changed} Showing the current card.` : changed
    else if (!found) view.value = { ...view.value, message: acknowledged }
    return found
  }

  async function sendExtra(request: PendingExtra): Promise<boolean> {
    const mine = ++epoch
    busy.value = true
    error.value = ''
    try {
      const result = await post<ExtraResult>('extra', request)
      if (mine !== epoch) return false
      const step = extendedPractice(view.value, result)
      pendingExtra.value = null
      busy.value = false
      if (step.reconcile) return await resync(false, 'Extra was saved. Press Daily to continue.')
      view.value = step.view
      return step.view.card !== null
    } catch (cause) {
      if (mine !== epoch) return false
      if (cause instanceof ApiError && cause.status < 500) pendingExtra.value = null
      error.value = pendingExtra.value
        ? `${String(cause)} Extra was not confirmed; retry it.`
        : String(cause)
      return false
    } finally {
      busy.value = false
    }
  }

  async function openExtra(lesson: string | null): Promise<boolean> {
    if (busy.value) return false
    lessonId = lesson
    // Every deliberate press gets its own operation ID; only Retry reuses one.
    pendingExtra.value = { operation_id: crypto.randomUUID(), lesson_id: lesson }
    return sendExtra(pendingExtra.value)
  }

  async function retryExtra(): Promise<boolean> {
    return pendingExtra.value && !busy.value ? sendExtra(pendingExtra.value) : false
  }

  async function act(path: string, body: object = {}, detailsOpen?: boolean) {
    const card = view.value.card
    if (!card || busy.value) return
    const mine = epoch
    error.value = ''
    try {
      const updated = await post<Card>(path, {
        view_token: card.view_token,
        phrase_id: card.phrase_id,
        ...body,
      })
      if (mine === epoch) view.value = updatedCard(view.value, updated, detailsOpen)
    } catch (cause) {
      if (mine !== epoch) return
      if (cause instanceof ApiError && cause.status === 409) await resync(true)
      else error.value = String(cause)
    }
  }

  async function send(request: PendingCompletion) {
    const mine = epoch
    busy.value = true
    error.value = ''
    try {
      const result = await post<CompletionResult>('complete', request)
      // The server kept this completion; a card the learner already left stays closed.
      if (mine !== epoch) return
      const step = completedPractice(view.value, request, result)
      pending.value = null
      busy.value = false
      if (step.reconcile) await resync(false, 'Your answer was saved. Press Daily to continue.')
      else view.value = step.view
    } catch (cause) {
      if (mine !== epoch) return
      if (cause instanceof ApiError && cause.status === 409) {
        pending.value = null
        busy.value = false
        await resync(true)
      } else error.value = `${String(cause)} The card is unchanged; retry to save it.`
    } finally {
      busy.value = false
    }
  }

  function closeCard() {
    epoch++
    view.value = initialView()
    pending.value = null
  }

  async function complete(outcome: Outcome) {
    if (busy.value || !view.value.card) return
    pending.value = pendingFor(view.value, outcome, crypto.randomUUID())
    await send(pending.value)
  }

  async function retry() {
    if (pending.value && !busy.value) await send(pending.value)
  }

  return {
    view,
    busy,
    error,
    pending,
    pendingExtra,
    openDaily,
    openExtra,
    retryExtra,
    openPhrase,
    reveal: (details = false) => act('reveal', {}, details || undefined),
    newCue: () => act('cue', { operation_id: crypto.randomUUID() }),
    hint: () => act('hint', { operation_id: crypto.randomUUID() }),
    toggleDetails: () => (view.value = { ...view.value, detailsOpen: !view.value.detailsOpen }),
    closeCard,
    complete,
    retry,
  }
}
