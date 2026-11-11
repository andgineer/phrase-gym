<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, useId, watch } from 'vue'
import { usePagedList } from '../composables/usePagedList'
import { onLearningDayChange } from '../learningDay'
import type { LessonChoice, LessonPage } from '../libraryTypes'

const selected = defineModel<string>({ default: '' })
const query = ref('')
const text = ref('')
const open = ref(false)
const active = ref(-1)
const root = ref<HTMLElement | null>(null)
const field = ref<HTMLInputElement | null>(null)
const listId = useId()
const optionId = (index: number) => `${listId}-${index}`
const lessons = usePagedList<LessonChoice, LessonPage>(() => {
  const params = new URLSearchParams({ query: query.value.trim() })
  if (selected.value) params.set('lesson_id', selected.value)
  return `/lessons?${params}`
})
const label = (lesson: LessonChoice) => `${lesson.title} · ${lesson.source_label}`
const choices = computed(() => [
  { id: '', title: 'All lessons', source: '' },
  ...lessons.items.value.map((lesson) => ({
    id: lesson.id,
    title: lesson.title,
    source: lesson.source_label,
  })),
])
const current = computed(() => {
  if (!selected.value) return null
  const page = lessons.page.value
  if (page?.selected?.id === selected.value) return page.selected
  return lessons.items.value.find((lesson) => lesson.id === selected.value) ?? null
})
const shown = computed(() => (current.value ? label(current.value) : ''))
const counts = computed(() => (selected.value ? current.value : lessons.page.value?.all_lessons))
const empty = computed(() => lessons.page.value?.all_lessons.phrase_count === 0)
const noMatches = computed(
  () => query.value.trim() && lessons.loaded.value && !lessons.items.value.length,
)
const summary = computed(() => {
  const value = counts.value
  if (!value) return ''
  if (!value.phrase_count) return 'No phrases in this lesson'
  if (value.due_count || value.new_count) return `${value.due_count} due · ${value.new_count} new`
  return value.next_due
    ? `Nothing waiting today · Next due: ${value.next_due}`
    : 'Nothing waiting today'
})
let debounce: number | undefined

function openList() {
  if (open.value) return
  open.value = true
  text.value = ''
  if (field.value) field.value.value = ''
  active.value = choices.value.findIndex((choice) => choice.id === selected.value)
}

function close() {
  open.value = false
  active.value = -1
  text.value = shown.value
  query.value = ''
}

function choose(index: number, keepFocus: boolean) {
  const choice = choices.value[index]
  if (!choice) return
  selected.value = choice.id
  close()
  if (!keepFocus) field.value?.blur()
}

function move(step: number) {
  const last = choices.value.length - 1
  active.value = Math.min(Math.max(active.value + step, 0), last)
  void nextTick(() =>
    document.getElementById(optionId(active.value))?.scrollIntoView({ block: 'nearest' }),
  )
}

function key(event: KeyboardEvent) {
  const typing = event.key.length === 1 || event.key === 'Backspace' || event.key === 'Delete'
  if (!open.value && typing && !event.ctrlKey && !event.metaKey && !event.altKey) openList()
  else if (event.key === 'ArrowDown') {
    event.preventDefault()
    if (open.value) move(1)
    else openList()
  } else if (event.key === 'ArrowUp' && open.value) {
    event.preventDefault()
    move(-1)
  } else if (event.key === 'Enter' && open.value && active.value >= 0) {
    event.preventDefault()
    choose(active.value, true)
  } else if (event.key === 'Escape' && open.value) {
    event.preventDefault()
    close()
  }
}

function filter() {
  openList()
  query.value = text.value
  // Until the matching lessons arrive, Enter must not pick a lesson from the previous list.
  active.value = -1
}

function leave(event: FocusEvent) {
  if (open.value && !root.value?.contains(event.relatedTarget as Node | null)) close()
}

// Touch WebKit keeps focus in the field when a non-focusable area is tapped.
function pressOutside(event: PointerEvent) {
  if (!open.value || root.value?.contains(event.target as Node | null)) return
  close()
  field.value?.blur()
}

watch(shown, (value) => {
  if (!open.value) text.value = value
})
watch(query, () => {
  window.clearTimeout(debounce)
  debounce = window.setTimeout(async () => {
    const wanted = query.value
    await lessons.reload()
    if (open.value && wanted.trim() && wanted === query.value)
      active.value = lessons.items.value.length ? 1 : -1
  }, 200)
})
onMounted(() => {
  document.addEventListener('pointerdown', pressOutside)
  void lessons.reload()
})
onLearningDayChange(() => void lessons.refresh())
onUnmounted(() => {
  document.removeEventListener('pointerdown', pressOutside)
  window.clearTimeout(debounce)
})
defineExpose({ reload: lessons.reload })
</script>

<template>
  <div v-if="empty" class="lesson-picker">
    <p>Add your first phrase</p>
    <p class="muted">
      Type a phrase or word into the Coach field, or ask your AI agent to add a course batch.
    </p>
  </div>
  <div v-else ref="root" class="lesson-picker" @focusout="leave">
    <div class="lesson-combo">
      <input
        ref="field"
        v-model="text"
        type="text"
        role="combobox"
        aria-label="Lesson"
        aria-autocomplete="list"
        :aria-expanded="open"
        :aria-controls="listId"
        :aria-activedescendant="open && active >= 0 ? optionId(active) : undefined"
        :placeholder="open ? 'Type to filter lessons' : 'All lessons — select one'"
        autocomplete="off"
        spellcheck="false"
        @focus="openList"
        @click="openList"
        @keydown="key"
        @input="filter"
      />
      <span class="combo-arrow" aria-hidden="true">▾</span>
      <div v-show="open" class="combo-panel" @mousedown.prevent>
        <ul :id="listId" role="listbox" aria-label="Lessons">
          <li
            v-for="(choice, index) in choices"
            :id="optionId(index)"
            :key="choice.id"
            role="option"
            :aria-selected="choice.id === selected"
            :class="{ active: index === active }"
            @click="choose(index, false)"
            @mousemove="active = index"
          >
            {{ choice.title
            }}<span v-if="choice.source" class="combo-source"> · {{ choice.source }}</span>
          </li>
        </ul>
        <p v-if="noMatches" class="muted">No matches</p>
        <button
          v-if="lessons.nextOffset.value !== null"
          type="button"
          class="quiet"
          @click="lessons.more"
        >
          More lessons
        </button>
      </div>
    </div>
    <p v-if="summary" class="muted" data-testid="lesson-counts">{{ summary }}</p>
    <p v-if="lessons.error.value" role="alert" class="error">{{ lessons.error.value }}</p>
  </div>
</template>
