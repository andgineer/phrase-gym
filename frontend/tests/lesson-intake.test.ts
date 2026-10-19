import { IDBFactory } from 'fake-indexeddb'
import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { inboxDraft } from '../src/inboxDraft'
import { useLessonIntake } from '../src/composables/useLessonIntake'
import { onboardingMessages } from '../src/onboardingMessages'
import { connectionRetrying, serverUnavailable } from '../src/api'

const mounted = vi.hoisted(() => [] as (() => Promise<void>)[])
vi.mock('vue', async () => ({
  ...(await vi.importActual('vue')),
  onMounted: (callback: () => Promise<void>) => mounted.push(callback),
}))
beforeEach(() => {
  mounted.length = 0
  vi.stubGlobal('indexedDB', new IDBFactory())
})
afterEach(() => {
  vi.unstubAllGlobals()
  vi.clearAllMocks()
  serverUnavailable.value = connectionRetrying.value = false
})
async function intake() {
  const model = useLessonIntake(() => onboardingMessages('en'))
  await mounted.at(-1)!()
  model.title.value = 'Opening a meeting'
  model.phrase.value = "Let's get started."
  model.context.value = 'All participants are present.'
  model.performer.value = 'Synthetic'
  model.file.value = new File(['synthetic-bytes'], 'reference.wav')
  return model
}

test('lost acknowledgement and reload retry the persisted payload without another audio upload', async () => {
  const bodies: string[] = []
  let uploads = 0
  const receipt = {
    generation_id: 'generation',
    submission_id: 'submission',
    generation: 1,
    namespace: 'test',
    profile_revision: 1,
    guide_version: 'test',
    received_at: '2026-10-19T10:00:00Z',
  }
  vi.stubGlobal(
    'fetch',
    vi.fn(async (path: string, options: RequestInit) => {
      const saved = await inboxDraft()
      expect(saved).toBeDefined()
      if (path.endsWith('/media')) {
        uploads++
        return Response.json({ id: 'audio-id', sha256: 'audio-sha256' })
      }
      if (options?.method === 'POST') {
        bodies.push(String(options.body))
        expect(saved?.submission).toEqual(JSON.parse(String(options.body)))
        if (bodies.length === 1) throw new Error('Synthetic lost response')
        return Response.json(receipt)
      }
      return Response.json({ receipt, status: 'waiting', reason: null, publication: null })
    }),
  )
  const first = await intake()
  await first.submit()
  expect(first.error.value).toContain('Synthetic lost response')
  expect(first.draft.value?.receipt).toBeUndefined()
  expect((await inboxDraft())?.submission.phrases[0]?.reference_audio_id).toBe('audio-id')
  serverUnavailable.value = false
  const reloaded = await intake()
  await Promise.all([reloaded.submit(), reloaded.submit()])
  expect(bodies).toHaveLength(2)
  expect(bodies[1]).toBe(bodies[0])
  expect(uploads).toBe(1)
  expect(reloaded.status.value?.status).toBe('waiting')
  expect((await inboxDraft())?.receipt).toEqual(receipt)
})

test('local storage refusal preserves editable input and sends no request', async () => {
  const model = await intake()
  const fetch = vi.fn()
  vi.stubGlobal('fetch', fetch)
  vi.stubGlobal('indexedDB', {
    open: () => {
      throw new Error('Synthetic quota refusal')
    },
  })
  await model.submit()
  expect(fetch).not.toHaveBeenCalled()
  expect(model.draft.value).toBeUndefined()
  expect(model.phrase.value).toBe("Let's get started.")
  expect(model.file.value?.name).toBe('reference.wav')
  expect(model.error.value).toContain('Could not save input on this device')
})

function accepted(id: string) {
  return {
    generation_id: id,
    submission_id: id,
    generation: 1,
    namespace: 'test',
    profile_revision: 1,
    guide_version: 'test',
    received_at: '2026-10-19T10:00:00Z',
  }
}
function successfulFetch(path: string, options?: RequestInit) {
  if (path.endsWith('/media')) return Response.json({ id: 'audio-id', sha256: 'audio-sha256' })
  if (options?.method === 'POST') {
    return Response.json(accepted(JSON.parse(String(options.body)).submission_id))
  }
  return Response.json({ status: 'waiting', reason: null, publication: null })
}

test('a stale empty instance cannot replace an unconfirmed delivery or send its new form', async () => {
  const first = await intake()
  const stale = await intake()
  const fetch = vi.fn(async (path: string, options?: RequestInit) => {
    if (path.endsWith('/inbox/lessons')) throw new Error('Synthetic lost response')
    return successfulFetch(path, options)
  })
  vi.stubGlobal('fetch', fetch)
  await first.submit()
  const original = await inboxDraft()
  serverUnavailable.value = false
  stale.title.value = 'A different lesson'
  stale.file.value = new File(['different-bytes'], 'different.wav')
  await stale.submit()
  expect(await inboxDraft()).toEqual(original)
  expect(fetch).toHaveBeenCalledTimes(2)
  expect(stale.draft.value).toEqual(original)
  expect(stale.error.value).toContain('changed in another tab')
  expect(stale.title.value).toBe('A different lesson')
  const reloaded = await intake()
  vi.stubGlobal(
    'fetch',
    vi.fn(async (path: string, options?: RequestInit) => successfulFetch(path, options)),
  )
  await reloaded.submit()
  expect(reloaded.draft.value?.submission).toEqual(original?.submission)
  expect(reloaded.draft.value?.upload).toEqual(original?.upload)
})

test.each(['media', 'receipt'])(
  'late %s cannot overwrite a replacement after explicit abandonment',
  async (phase) => {
    let release!: (response: Response) => void
    let started!: () => void
    const pending = new Promise<void>((resolve) => {
      started = resolve
    })
    let delayed = false
    let oldId = ''
    vi.stubGlobal(
      'fetch',
      vi.fn(async (path: string, options?: RequestInit) => {
        const target = phase === 'media' ? path.endsWith('/media') : path.endsWith('/inbox/lessons')
        if (target && !delayed) {
          delayed = true
          oldId = (await inboxDraft())!.submission.submission_id
          started()
          return new Promise<Response>((resolve) => {
            release = resolve
          })
        }
        return successfulFetch(path, options)
      }),
    )
    const first = await intake()
    const sending = first.submit()
    await pending
    const other = await intake()
    const stale = await intake()
    await other.clear()
    other.title.value = 'Replacement'
    other.phrase.value = 'Another phrase.'
    other.context.value = 'Another situation'
    other.file.value = new File(['replacement-bytes'], 'replacement.wav')
    await other.submit()
    const replacement = await inboxDraft()
    expect(replacement?.submission.submission_id).not.toBe(oldId)
    expect(replacement?.receipt).toBeDefined()
    release(
      phase === 'media'
        ? Response.json({ id: 'late-media', sha256: 'late-sha' })
        : Response.json(accepted(oldId)),
    )
    await sending
    expect(await inboxDraft()).toEqual(replacement)
    expect(first.draft.value).toEqual(replacement)
    expect(first.error.value).toContain('changed in another tab')
    await stale.clear()
    expect(await inboxDraft()).toEqual(replacement)
    expect(stale.draft.value).toEqual(replacement)
    expect(stale.error.value).toContain('changed in another tab')
  },
)

test.each(['failed', 'cancelled', 'superseded'])(
  'local abandonment of accepted %s work permits new intake without mutating it',
  async (state) => {
    const fetch = vi.fn(async (path: string, options?: RequestInit) => {
      if (path.endsWith('/retry'))
        return Response.json(
          { detail: 'Only failed jobs below the attempt limit may retry' },
          { status: 409 },
        )
      if (!options?.method)
        return Response.json({
          status: state,
          reason: 'Synthetic terminal result',
          publication: null,
        })
      return successfulFetch(path, options)
    })
    vi.stubGlobal('fetch', fetch)
    const model = await intake()
    await model.submit()
    const original = await inboxDraft()
    expect(model.status.value?.status).toBe(state)
    if (state === 'failed') {
      await model.retryPreparation()
      expect(model.error.value).toContain('attempt limit')
      expect(model.status.value?.status).toBe('failed')
    }
    const calls = fetch.mock.calls.length
    await model.clear()
    expect(fetch.mock.calls).toHaveLength(calls)
    expect(await inboxDraft()).toBeUndefined()
    const reloaded = await intake()
    expect(reloaded.draft.value).toBeUndefined()
    await reloaded.submit()
    expect(reloaded.draft.value?.submission.submission_id).not.toBe(
      original?.submission.submission_id,
    )
    expect(reloaded.draft.value?.receipt).toBeDefined()
  },
)
