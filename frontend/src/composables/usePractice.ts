import { ref } from 'vue'
import { ApiError, api } from '../api'
import {
  completedPractice,
  initialView,
  openedState,
  pendingFor,
  updatedCard,
  type PendingCompletion,
} from '../practiceReducer'
import type { Card, CompletionResult, Outcome, PracticeState } from '../studyTypes'

const changedElsewhere =
  'This card changed in another tab or a new day began. Showing the current card.'

export function usePractice() {
  const view = ref(initialView())
  const busy = ref(false)
  const error = ref('')
  const pending = ref<PendingCompletion | null>(null)
  let lessonId: string | null = null

  function post<T>(path: string, body: object) {
    return api<T>(`/practice/${path}`, { method: 'POST', body: JSON.stringify(body) })
  }

  async function openDaily(lesson: string | null = lessonId) {
    lessonId = lesson
    busy.value = true
    error.value = ''
    try {
      view.value = openedState(await post<PracticeState>('daily', { lesson_id: lesson }))
      pending.value = null
    } catch (cause) {
      error.value = String(cause)
    } finally {
      busy.value = false
    }
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
      if (cause instanceof ApiError && cause.status === 409) await reconcile()
      else error.value = String(cause)
    }
  }

  async function reconcile() {
    await openDaily()
    if (!error.value) error.value = changedElsewhere
  }

  async function send(request: PendingCompletion) {
    busy.value = true
    error.value = ''
    try {
      const result = await post<CompletionResult>('complete', request)
      const step = completedPractice(view.value, request, result)
      pending.value = null
      if (step.reconcile) await openDaily()
      else view.value = step.view
    } catch (cause) {
      if (cause instanceof ApiError && cause.status === 409) {
        pending.value = null
        busy.value = false
        await reconcile()
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
    openDaily,
    reveal: (details = false) => act('reveal', {}, details || undefined),
    newCue: () => act('cue', { operation_id: crypto.randomUUID() }),
    hint: () => act('hint', { operation_id: crypto.randomUUID() }),
    toggleDetails: () => (view.value = { ...view.value, detailsOpen: !view.value.detailsOpen }),
    complete,
    retry,
  }
}
