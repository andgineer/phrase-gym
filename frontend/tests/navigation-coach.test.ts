import { expect, it } from 'vitest'
import { addressFromHash, hashFor } from '../src/composables/useAppNavigation'
import { coachText } from '../src/composables/useCoachSubmission'

it('reads the learning language and page from the address', () => {
  for (const page of ['coach', 'library', 'settings'] as const) {
    expect(addressFromHash(`#de/${page}`)).toEqual({ language: 'de', page })
    expect(addressFromHash(hashFor('de', page))).toEqual({ language: 'de', page })
    // Bookmarks from before learning languages open English.
    expect(addressFromHash(`#${page}`)).toEqual({ language: 'en', page })
  }
  expect(addressFromHash('#de')).toEqual({ language: 'de', page: 'entry' })
  expect(addressFromHash('#en')).toEqual({ language: 'en', page: 'entry' })
  expect(addressFromHash('#de/other')).toEqual({ language: 'de', page: 'entry' })
  for (const hash of ['', '#practice', '#other', '#fr', '#fr/library', '#library/de'])
    expect(addressFromHash(hash)).toEqual({ language: 'en', page: 'entry' })
})

it('keeps an open card at its language’s Practice address', () => {
  expect(hashFor('de', 'card')).toBe('#de')
  expect(hashFor('en', 'entry')).toBe('#en')
  expect(hashFor('en', 'settings')).toBe('#en/settings')
})

it('keeps pasted Coach material in the existing single-line request shape', () => {
  expect(coachText('  by and large\r\nIt worked.\n\nMostly.  ')).toBe(
    'by and large It worked. Mostly.',
  )
  expect(coachText(' \r\n ')).toBe('')
})
