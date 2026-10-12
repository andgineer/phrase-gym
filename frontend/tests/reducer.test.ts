import { describe, expect, it } from 'vitest'
import { initialCard, reduceCard, referenceCard } from '../src/reducer'
import { language } from '../src/api'

describe('original response evidence', () => {
  it('snapshots silent recall before answer reveal', () => {
    const answer = reduceCard(initialCard(), { type: 'answer' })
    expect(answer.revealed).toBe(true)
    expect(answer.originalResponse).toEqual({ modality: 'self_report', supported: false })
    expect(reduceCard(answer, { type: 'explain' }).originalResponse).toEqual(
      answer.originalResponse,
    )
  })
  it('retains support from lookup and never rewrites the original response', () => {
    const lookup = referenceCard()
    expect(lookup.originalResponse).toBeNull()
    const supported = reduceCard(lookup, { type: 'respond', modality: 'recorded' })
    expect(supported.originalResponse?.supported).toBe(true)
    const rerecord = reduceCard(supported, { type: 'respond', modality: 'self_report' })
    expect(rerecord.originalResponse).toEqual(supported.originalResponse)
  })
  it('does not retroactively support a captured response when explanation follows', () => {
    const captured = reduceCard(initialCard(), { type: 'respond', modality: 'recorded' })
    expect(reduceCard(captured, { type: 'explain' }).originalResponse?.supported).toBe(false)
  })
  it('rejects invalid language routes without substitution', () => {
    expect(language('sr')).toBe('sr')
    expect(language('unknown')).toBeNull()
    expect(language(['en', 'sr'])).toBeNull()
  })
})
