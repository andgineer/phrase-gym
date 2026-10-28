import { beforeEach, expect, it, vi } from 'vitest'
import { api } from '../src/api'
import { useStudy } from '../src/composables/useStudy'

vi.mock('../src/api', () => ({ api: vi.fn() }))

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
    '/study/history',
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
  const conflict = Object.assign(new Error('Attempt already completed'), { status: 409 })
  vi.mocked(api)
    .mockRejectedValueOnce(conflict)
    .mockResolvedValueOnce({ card: null })
    .mockResolvedValueOnce([])

  await study.rate('exposed-attempt', 'recalled', 'silent')

  expect(study.current.value?.card).toBeNull()
  expect(study.error.value).toContain('skipped without a grade')
  expect(vi.mocked(api).mock.calls.map(([path]) => path)).toEqual([
    '/attempts/exposed-attempt/review',
    '/study/current',
    '/study/history',
  ])
})
