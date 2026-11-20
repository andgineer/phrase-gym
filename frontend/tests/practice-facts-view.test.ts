import { beforeEach, expect, it, vi } from 'vitest'
import { computed, createSSRApp, ref, shallowRef } from 'vue'
import { renderToString } from 'vue/server-renderer'
import PracticeFactsView from '../src/components/PracticeFacts.vue'
import { usePracticeFacts } from '../src/composables/usePracticeFacts'
import { usePagedList } from '../src/composables/usePagedList'
import type { PracticeFacts, StudiedPage, StudiedPhrase } from '../src/factsTypes'
import { facts, item } from './facts-fixture'

vi.mock('../src/composables/usePracticeFacts')

const pendingUploads = ref(0)

function state(value: PracticeFacts | null) {
  return {
    facts: shallowRef(value),
    error: ref(''),
    listing: ref(false),
    studied: usePagedList<StudiedPhrase, StudiedPage>(() => '/unused'),
    pendingUploads: computed(() => pendingUploads.value),
    refresh: vi.fn(async () => {}),
    toggleList: vi.fn(async () => {}),
    dismiss: vi.fn(async () => {}),
  }
}
let model = state(facts())
const render = (message = '') => renderToString(createSSRApp(PracticeFactsView, { message }))
const emptyTotals = { ...facts().today }
for (const field of Object.keys(emptyTotals)) emptyTotals[field as keyof typeof emptyTotals] = 0

beforeEach(() => {
  pendingUploads.value = 0
  model = state(facts())
  vi.mocked(usePracticeFacts).mockReturnValue(model)
})

it('renders each facts group from its own server totals', async () => {
  const html = await render()
  for (const text of [
    'Today — 20 recalls / 5 new',
    '18 Good · 2 Hard',
    '30 recordings made',
    '🔥 8-day practice streak',
    'Personal best: 12 days',
    'This week: 2 of 3 days',
    'Progress — 3 days',
    '45 recalls · 112 new phrases',
    '39 Good · 6 Hard',
    'Total — 131 phrases',
    '72 due now · 39 later · 20 new',
    '12 in long-term memory',
  ])
    expect(html).toContain(text)
  expect(html).not.toContain('Last seven days')
})

it('hides the whole panel when there is no practice, target, message or pending upload', async () => {
  model.facts.value = facts({
    today: emptyTotals,
    work: { ...emptyTotals, practice_days: 0 },
    week: { practice_days: 0, target: null },
  })
  expect(await render()).not.toContain('aria-label="Practice facts"')
  pendingUploads.value = 1
  expect(await render()).toContain('1 recording waiting to upload from this device')
})

it('hides unpractised and empty groups without hiding a chosen weekly target', async () => {
  model.facts.value = facts({
    today: emptyTotals,
    work: { ...emptyTotals, practice_days: 0 },
    week: { practice_days: 0, target: 3 },
  })
  const html = await render()
  expect(html).toContain('This week: 0 of 3 days')
  for (const id of ['today-title', 'streak', 'best-streak', 'progress-title', 'total-title'])
    expect(html).not.toContain(`data-testid="${id}"`)
})

it('keeps earlier practice visible after a missed day but hides an empty Today and paused target', async () => {
  model.facts.value = facts({
    today: emptyTotals,
    streak: { current: 0, best: 2, recent_days: [] },
    week: { practice_days: 1, target: null },
    library: { phrase_count: 6, due_count: 3, new_count: 3, later_count: 0, long_term_count: 0 },
  })
  const html = await render()
  for (const text of [
    'No current practice streak',
    'Personal best: 2 days',
    'Progress — 3 days',
    '3 due now · 3 new',
  ])
    expect(html).toContain(text)
  for (const id of ['today-title', 'week', 'long-term'])
    expect(html).not.toContain(`data-testid="${id}"`)
})

it('renders pending device uploads separately from saved recording totals', async () => {
  pendingUploads.value = 1
  model.facts.value = facts({ today: { ...emptyTotals, recordings: 1 } })
  const html = await render()
  expect(html).toContain('1 recording waiting to upload from this device')
  expect(html).toContain('1 recording made')
  expect(html).toContain('data-testid="today-title"')
  expect(html).not.toContain('data-testid="today-ratings"')
})

it('renders studied history, deleted snapshots and pagination only while expanded', async () => {
  model.studied.items.value = [
    item({}),
    item({
      completion_id: 'deleted-completion',
      phrase_id: null,
      mode: 'extra',
      rating: 'not_yet',
    }),
  ]
  model.studied.nextOffset.value = 5
  expect(await render()).not.toContain('aria-label="Today’s phrases"')
  model.listing.value = true
  const html = await render()
  expect(html).toContain('aria-label="Today’s phrases"')
  expect(html).toContain('Hard · Daily · Small talk')
  expect(html).toContain('Again · Extra · Small talk · deleted')
  expect(html).toContain('>More</button>')
  model.studied.nextOffset.value = null
  expect(await render()).not.toContain('>More</button>')
})

it('shows a completion message only after facts resolve or fail', async () => {
  const message = "Today's practice is complete."
  model.facts.value = null
  expect(await render(message)).not.toContain('entry-message')
  model.error.value = 'Error: Request failed (503)'
  const html = await render(message)
  expect(html).toContain('entry-message')
  expect(html).toContain('Request failed (503)')
})
