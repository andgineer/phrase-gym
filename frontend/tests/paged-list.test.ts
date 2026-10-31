import { beforeEach, expect, test, vi } from 'vitest'
import { api } from '../src/api'
import { usePagedList } from '../src/composables/usePagedList'

vi.mock('../src/api', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../src/api')>()),
  api: vi.fn(),
}))

const total = 120
let version = 'old'
function serve(path: string) {
  const query = new URLSearchParams(path.split('?')[1])
  const offset = Number(query.get('offset'))
  const limit = Number(query.get('limit'))
  const end = Math.min(offset + limit, total)
  const items = Array.from({ length: end - offset }, (_, index) => `${version}-${offset + index}`)
  return { items, next_offset: end < total ? end : null }
}

beforeEach(() => {
  version = 'old'
  vi.mocked(api).mockReset()
  vi.mocked(api).mockImplementation(async (path) => serve(path))
})

const requested = () => vi.mocked(api).mock.calls.map(([path]) => path)

test('refresh reloads every loaded item in pages the server accepts', async () => {
  const list = usePagedList<string>(() => '/lessons?query=')
  await list.reload()
  for (let page = 1; page < 6; page++) await list.more()
  expect(list.items.value).toHaveLength(120)
  expect(list.nextOffset.value).toBeNull()
  vi.mocked(api).mockClear()
  version = 'new'
  await list.refresh()
  expect(requested()).toEqual([
    '/lessons?query=&offset=0&limit=100',
    '/lessons?query=&offset=100&limit=20',
  ])
  expect(list.items.value).toHaveLength(120)
  expect(list.items.value.at(-1)).toBe('new-119')
  expect(list.nextOffset.value).toBeNull()
})

test('a slower earlier refresh cannot replace a newer reload', async () => {
  const list = usePagedList<string>(() => '/lessons?query=')
  await list.reload()
  let release!: () => void
  const held = new Promise<void>((resolve) => (release = resolve))
  vi.mocked(api).mockImplementationOnce(async (path) => {
    await held
    return serve(path)
  })
  const slow = list.refresh()
  version = 'newer'
  await list.reload()
  release()
  await slow
  expect(list.items.value[0]).toBe('newer-0')
  expect(list.items.value).toHaveLength(20)
})
