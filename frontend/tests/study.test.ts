import { beforeEach, expect, it, vi } from 'vitest'
import { ApiError, api } from '../src/api'
import { useStudy } from '../src/composables/useStudy'
import type { Study } from '../src/studyTypes'

vi.mock('../src/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../src/api')>()),
  api: vi.fn(),
}))

beforeEach(() => vi.mocked(api).mockReset())

it('ignores repeated Next while saving audio and completes the original attempt once', async () => {
  const study = useStudy()
  let release!: () => void
  const pendingAudio = new Promise<void>((resolve) => {
    release = resolve
  })
  const prepare = vi.fn(() => pendingAudio)
  vi.mocked(api).mockResolvedValue({})

  const first = study.complete('original-attempt', prepare)
  expect(study.completing.value).toBe(true)
  await study.complete('another-attempt', prepare)
  expect(prepare).toHaveBeenCalledTimes(1)
  expect(api).not.toHaveBeenCalled()

  release()
  await first
  expect(vi.mocked(api).mock.calls.map(([path]) => path)).toEqual([
    '/attempts/original-attempt/complete',
    '/study/current',
  ])
  expect(study.completing.value).toBe(false)
})

it.each(['preparation', 'request'])(
  'shows a %s failure and allows another completion',
  async (failure) => {
    const study = useStudy()
    const prepare = vi.fn().mockResolvedValue(undefined)
    if (failure === 'preparation') prepare.mockRejectedValueOnce(new Error('Save failed'))
    else vi.mocked(api).mockRejectedValueOnce(new Error('Save failed'))

    await study.complete('attempt', prepare)
    expect(study.error.value).toContain('Save failed')
    expect(study.completing.value).toBe(false)

    vi.mocked(api).mockResolvedValue({})
    await study.complete('attempt', prepare)
    expect(study.error.value).toBe('')
    expect(study.completing.value).toBe(false)
  },
)

it('refreshes a review invalidated by answer exposure instead of leaving rating buttons active', async () => {
  const study = useStudy()
  const conflict = new ApiError('Attempt already completed', 409)
  vi.mocked(api).mockRejectedValueOnce(conflict).mockResolvedValueOnce({ card: null })

  await study.rate('exposed-attempt', 'recalled', 'silent')

  expect(study.current.value?.card).toBeNull()
  expect(study.error.value).toContain('skipped without a grade')
  expect(vi.mocked(api).mock.calls.map(([path]) => path)).toEqual([
    '/attempts/exposed-attempt/review',
    '/study/current',
  ])
})

it('refreshes the rated card recordings in capture order and ignores a deleted phrase', async () => {
  const study = useStudy()
  study.current.value = {
    done: false,
    total: 1,
    studied: 0,
    card: { id: 'phrase', attempt_id: 'attempt', recordings: [] },
  } as unknown as Study
  vi.mocked(api)
    .mockResolvedValueOnce({})
    .mockResolvedValueOnce({ items: [{ id: 'newer' }, { id: 'older' }], next_offset: null })
  await study.rate('attempt', 'with_effort', 'silent')
  expect(study.current.value?.card?.recordings.map((take) => take.id)).toEqual(['older', 'newer'])
  expect(vi.mocked(api).mock.calls.at(-1)?.[0]).toBe('/phrases/phrase/recordings?limit=100')

  vi.mocked(api).mockRejectedValueOnce(new ApiError('Phrase not found', 404))
  await study.refreshRecordings()
  expect(study.error.value).toBe('')
})
