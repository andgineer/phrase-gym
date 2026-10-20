import { onMounted, ref } from 'vue'
import { api, ApiError, requireServer } from '../api'
import type { components } from '../../tests/rust-contract/api.generated'
import { audioInput, inboxDraft, type InboxDraft } from '../inboxDraft'
import type { OnboardingMessages } from '../onboardingMessages'

type Schemas = components['schemas']
export function useLessonIntake(messages: () => OnboardingMessages) {
  const draft = ref<InboxDraft>()
  const status = ref<Schemas['LessonStatus']>()
  const busy = ref(false)
  const error = ref('')
  const ready = ref(false)
  const title = ref('')
  const phrase = ref('')
  const context = ref('')
  const note = ref('')
  const locale = ref('en-US')
  const performer = ref('')
  const file = ref<File>()
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
  async function acceptAudio(selected?: File) {
    if (busy.value || draft.value || !selected) return
    file.value = selected
    if (
      ![title.value, phrase.value, context.value, performer.value, locale.value].every((s) =>
        s.trim(),
      )
    ) {
      error.value = messages().missingFields
      return
    }
    await submit()
  }
  async function submit() {
    if (busy.value || !ready.value) return
    busy.value = true
    error.value = ''
    try {
      requireServer()
      if (!draft.value) {
        let audio: Awaited<ReturnType<typeof audioInput>>
        try {
          if (!file.value) throw new Error('audio')
          audio = await audioInput(file.value)
        } catch {
          throw new Error(messages().invalidAudio)
        }
        const id = crypto.randomUUID()
        const input: InboxDraft = {
          audio_sha256: audio.sha256,
          upload: {
            mime_type: audio.mime,
            audio_base64: audio.base64,
            metadata: {
              language: 'en',
              text: phrase.value.trim(),
              role: 'reference',
              origin: 'course',
              performer: performer.value.trim(),
              source: 'browser-supplied-lesson',
              locale: locale.value.trim(),
              preparation_version: 'supplied-v1',
            },
          },
          submission: {
            submission_id: id,
            item_id: id,
            expected_generation: null,
            language: 'en',
            title: title.value.trim(),
            context: context.value.trim(),
            note: note.value.trim(),
            phrases: [
              {
                source_id: 'phrase',
                text: phrase.value.trim(),
                reference_audio_id: '00000000-0000-0000-0000-000000000000',
                audio_sha256: '',
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
        if (!input.submission.phrases[0]!.audio_sha256) {
          const asset = await api<Schemas['MediaAsset']>('/media', {
            method: 'POST',
            body: JSON.stringify(input.upload),
          })
          if (asset.sha256 !== input.audio_sha256)
            throw new Error(
              'Uploaded reference hash does not match the retained original. Retry without changing the saved input.',
            )
          input.submission.phrases[0]!.reference_audio_id = asset.id
          input.submission.phrases[0]!.audio_sha256 = asset.sha256
          input = await save(input, input.submission.submission_id)
        }
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
      title.value = phrase.value = context.value = note.value = ''
      file.value = undefined
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
    title,
    phrase,
    context,
    note,
    locale,
    performer,
    file,
    submit,
    acceptAudio,
    refresh,
    retryPreparation,
    restore,
    clear,
  }
}
