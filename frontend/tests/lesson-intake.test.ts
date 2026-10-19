import { afterEach, beforeEach, expect, test, vi } from 'vitest'
import { inboxDraft, type InboxDraft } from '../src/inboxDraft'
import { useLessonIntake } from '../src/composables/useLessonIntake'
import { onboardingMessages } from '../src/onboardingMessages'
import { connectionRetrying, serverUnavailable } from '../src/api'

const mounted = vi.hoisted(() => [] as (() => Promise<void>)[])
vi.mock('vue', async () => ({
  ...(await vi.importActual('vue')),
  onMounted: (callback: () => Promise<void>) => mounted.push(callback),
}))
vi.mock('../src/inboxDraft', async () => ({
  ...(await vi.importActual('../src/inboxDraft')),
  inboxDraft: vi.fn(),
}))
let saved: InboxDraft | undefined
beforeEach(() => {
  saved = undefined
  mounted.length = 0
  vi.mocked(inboxDraft).mockImplementation(async (value) => {
    if (value !== undefined) saved = value === null ? undefined : structuredClone(value)
    return structuredClone(saved)
  })
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
  expect(saved?.submission.phrases[0]?.reference_audio_id).toBe('audio-id')
  serverUnavailable.value = false
  const reloaded = await intake()
  await Promise.all([reloaded.submit(), reloaded.submit()])
  expect(bodies).toHaveLength(2)
  expect(bodies[1]).toBe(bodies[0])
  expect(uploads).toBe(1)
  expect(reloaded.status.value?.status).toBe('waiting')
  expect(saved?.receipt).toEqual(receipt)
})

test('local storage refusal preserves editable input and sends no request', async () => {
  const model = await intake()
  const fetch = vi.fn()
  vi.stubGlobal('fetch', fetch)
  vi.mocked(inboxDraft).mockRejectedValue(new Error('Synthetic quota refusal'))
  await model.submit()
  expect(fetch).not.toHaveBeenCalled()
  expect(model.draft.value).toBeUndefined()
  expect(model.phrase.value).toBe("Let's get started.")
  expect(model.file.value?.name).toBe('reference.wav')
  expect(model.error.value).toContain('Could not save input on this device')
})
