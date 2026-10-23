import { expect, test } from 'vitest'
import { targetLabel } from '../src/targetLabel'
import type { Phrase } from '../src/api'

const phrase = (english: unknown, text: string) =>
  ({ content: { text }, english }) as unknown as Phrase

test('word senses are listed by word and sense while phrases keep their text', () => {
  const word = phrase(
    {
      task: {
        kind: 'word_sense',
        word: 'impediment',
        sense: 'something that makes progress difficult',
        contexts: [],
      },
    },
    'A lack of funding became a major impediment to the project.',
  )
  expect(targetLabel(word)).toBe('impediment — something that makes progress difficult')
  expect(targetLabel(phrase({ task: null }, "Let's get started."))).toBe("Let's get started.")
  expect(
    targetLabel(
      phrase(
        { task: { kind: 'frame', pattern: 'Could we move it to {day}?', slots: [] } },
        'Could we move it to Friday?',
      ),
    ),
  ).toBe('Could we move it to Friday?')
  expect(targetLabel(phrase(undefined, 'Guten Morgen.'))).toBe('Guten Morgen.')
})
