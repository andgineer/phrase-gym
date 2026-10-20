import { onMounted, ref } from 'vue'
import { api, ApiError, requireServer } from '../api'
import type { components } from '../../tests/rust-contract/api.generated'
import { inboxDraft, type InboxDraft } from '../inboxDraft'
import type { OnboardingMessages } from '../onboardingMessages'

type Schemas = components['schemas']
export function useLessonIntake(messages: () => OnboardingMessages) {
  const draft = ref<InboxDraft>()
  const status = ref<Schemas['LessonStatus']>()
  const busy = ref(false)
  const error = ref('')
  const ready = ref(false)
  const phrase = ref('')
  const note = ref('')
  function failure(cause: unknown) {
    error.value =
      cause instanceof ApiError && [401, 403].includes(cause.status)
        ? draft.value?.receipt
          ? messages().statusUnavailable
          : messages().permission
        : String(cause)
  }
  async function save(input: InboxDraft, expectedId?: string) {
    const current = await inboxDraft(input, expectedId)
    draft.value = current
    if (current?.submission.submission_id !== input.submission.submission_id) {
      status.value = undefined
      throw new Error(messages().conflict)
    }
    return current
  }
  async function refresh() {
    if (!draft.value?.receipt || busy.value) return
    busy.value = true
    error.value = ''
    try {
      status.value = await api<Schemas['LessonStatus']>(
        `/inbox/lessons/${draft.value.receipt.generation_id}`,
      )
    } catch (cause) {
      failure(cause)
    } finally {
      busy.value = false
    }
  }
  async function submit() {
    if (busy.value || !ready.value) return
    busy.value = true
    error.value = ''
    try {
      if (!draft.value) {
        if (!phrase.value.trim()) throw new Error('Paste or type a phrase before sending.')
        const id = crypto.randomUUID()
        const input: InboxDraft = {
          submission: {
            submission_id: id,
            item_id: id,
            expected_generation: null,
            language: 'en',
            title: 'Inbox practice',
            context: '',
            note: note.value.trim(),
            phrases: [
              {
                source_id: 'phrase',
                text: phrase.value.trim(),
                reference_audio_id: null,
                audio_sha256: null,
                expected_revision: null,
              },
            ],
          },
        }
        try {
          await save(input)
        } catch (cause) {
          if (draft.value) throw cause
          throw new Error(messages().storage)
        }
      }
      requireServer()
      let input = await save(
        JSON.parse(JSON.stringify(draft.value)),
        draft.value!.submission.submission_id,
      )
      if (!input.receipt) {
        const receipt = await api<Schemas['LessonReceipt'] | null>(
          `/inbox/submissions/${input.submission.submission_id}`,
        )
        if (receipt) {
          input.receipt = receipt
          input = await save(input, input.submission.submission_id)
        }
      }
      if (!input.receipt) {
        input.receipt = await api<Schemas['LessonReceipt']>('/inbox/lessons', {
          method: 'POST',
          body: JSON.stringify(input.submission),
        })
        await save(input, input.submission.submission_id)
      }
    } catch (cause) {
      failure(cause)
    } finally {
      busy.value = false
    }
    if (draft.value?.receipt && !error.value) await refresh()
  }
  async function retryPreparation() {
    if (!draft.value?.receipt || busy.value) return
    busy.value = true
    error.value = ''
    try {
      status.value = await api<Schemas['LessonStatus']>(
        `/inbox/lessons/${draft.value.receipt.generation_id}/retry`,
        { method: 'POST' },
      )
    } catch (cause) {
      failure(cause)
    } finally {
      busy.value = false
    }
  }
  async function clear() {
    if (busy.value) return
    busy.value = true
    try {
      const expectedId = draft.value?.submission.submission_id
      draft.value = await inboxDraft(null, expectedId)
      status.value = undefined
      if (draft.value) throw new Error(messages().conflict)
      phrase.value = note.value = ''
      error.value = ''
    } catch (cause) {
      failure(cause)
    } finally {
      busy.value = false
    }
  }
  async function restore() {
    if (busy.value) return
    busy.value = true
    error.value = ''
    try {
      draft.value = await inboxDraft()
      ready.value = true
    } catch {
      error.value = messages().storage
    } finally {
      busy.value = false
    }
    await refresh()
  }
  onMounted(restore)
  return {
    draft,
    status,
    busy,
    error,
    ready,
    phrase,
    note,
    submit,
    refresh,
    retryPreparation,
    restore,
    clear,
  }
}
