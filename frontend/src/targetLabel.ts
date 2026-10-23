import type { Phrase } from './api'

// A word is found by the word itself; its first example sentence alone hides it in lists.
export function targetLabel(phrase: Phrase): string {
  const task = phrase.english?.task
  return task?.kind === 'word_sense' ? `${task.word} — ${task.sense}` : phrase.content.text
}
