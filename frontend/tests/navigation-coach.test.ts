import { expect, it } from 'vitest'
import { pageFromHash } from '../src/composables/useAppNavigation'
import { coachText } from '../src/composables/useCoachSubmission'

it('selects only supported pages from the URL', () => {
  for (const page of ['coach', 'library', 'settings']) {
    expect(pageFromHash(`#${page}`)).toBe(page)
  }
  for (const hash of ['', '#practice', '#other']) expect(pageFromHash(hash)).toBe('entry')
})

it('keeps pasted Coach material in the existing single-line request shape', () => {
  expect(coachText('  by and large\r\nIt worked.\n\nMostly.  ')).toBe(
    'by and large It worked. Mostly.',
  )
  expect(coachText(' \r\n ')).toBe('')
})
