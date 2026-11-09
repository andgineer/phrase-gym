import { describe, expect, it } from 'vitest'
import { returnDescription, returnSpan } from '../src/returnSpan'

describe('returnSpan', () => {
  it('uses days below a month', () => {
    expect([1, 8, 19, 30].map(returnSpan)).toEqual(['1d', '8d', '19d', '30d'])
  })

  it('uses months with one decimal below a year', () => {
    expect([31, 46, 77, 163, 364].map(returnSpan)).toEqual(['1m', '1.5m', '2.5m', '5.4m', '12m'])
  })

  it('uses years with one decimal from a year', () => {
    expect([365, 497, 1346].map(returnSpan)).toEqual(['1y', '1.4y', '3.7y'])
  })
})

describe('returnDescription', () => {
  it('names the exact return in words', () => {
    expect(returnDescription(1)).toBe('Back tomorrow')
    expect(returnDescription(46)).toBe('Back in 46 days')
  })
})
