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

const changedElsewhere =
  'This card changed in another tab or a new day began. Showing the current card.'
const practiceClosed = 'This Practice card changed in another tab or a new day began.'

export function usePractice() {
  const view = ref(initialView())
  const busy = ref(false)
  const error = ref('')
  const pending = ref<PendingCompletion | null>(null)
  const pendingExtra = ref<PendingExtra | null>(null)
  let lessonId: string | null = null

  function post<T>(path: string, body: object) {
    return api<T>(`/practice/${path}`, { method: 'POST', body: JSON.stringify(body) })
  }

  async function load(path: string, body: object) {
    busy.value = true
    error.value = ''
    try {
      view.value = openedState(await post<PracticeState>(path, body))
      pending.value = null
    } catch (cause) {
      error.value = String(cause)
    } finally {
      busy.value = false
    }
  }

  async function openDaily(lesson: string | null = lessonId) {
    lessonId = lesson
    await load('daily', { lesson_id: lesson })
  }

  const openPhrase = (phraseId: string) => load('phrase', { phrase_id: phraseId })

  // Practice never falls back to Daily; closing its card returns to the phrase's details.
  async function resync(notice = '') {
    if (view.value.card?.mode === 'practice') {
      view.value = initialView()
      pending.value = null
      error.value = notice && practiceClosed
      return
    }
    await openDaily()
    if (!error.value) error.value = notice
  }

  async function sendExtra(request: PendingExtra) {
    busy.value = true
    error.value = ''
    try {
      const step = extendedPractice(view.value, await post<ExtraResult>('extra', request))
      pendingExtra.value = null
      busy.value = false
      if (step.reconcile) await openDaily()
      else view.value = step.view
    } catch (cause) {
      if (cause instanceof ApiError && cause.status < 500) pendingExtra.value = null
      error.value = pendingExtra.value
        ? `${String(cause)} Extra was not confirmed; retry it.`
        : String(cause)
    } finally {
      busy.value = false
    }
  }

  async function openExtra(lesson: string | null) {
    if (busy.value) return
    lessonId = lesson
    // Every deliberate press gets its own operation ID; only Retry reuses one.
    pendingExtra.value = { operation_id: crypto.randomUUID(), lesson_id: lesson }
    await sendExtra(pendingExtra.value)
  }

  async function retryExtra() {
    if (pendingExtra.value && !busy.value) await sendExtra(pendingExtra.value)
  }

  async function act(path: string, body: object = {}, detailsOpen?: boolean) {
    const card = view.value.card
    if (!card || busy.value) return
    error.value = ''
    try {
      const updated = await post<Card>(path, {
        view_token: card.view_token,
        phrase_id: card.phrase_id,
        ...body,
      })
      view.value = updatedCard(view.value, updated, detailsOpen)
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 409) await resync(changedElsewhere)
      else error.value = String(cause)
    }
  }

  async function send(request: PendingCompletion) {
    busy.value = true
    error.value = ''
    try {
      const result = await post<CompletionResult>('complete', request)
      const step = completedPractice(view.value, request, result)
      pending.value = null
      busy.value = false
      if (step.reconcile) await resync()
      else view.value = step.view
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 409) {
        pending.value = null
        busy.value = false
        await resync(changedElsewhere)
      } else error.value = `${String(cause)} The card is unchanged; retry to save it.`
    } finally {
      busy.value = false
    }
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
    closeCard: () => (view.value = initialView()),
    complete,
    retry,
  }
}
