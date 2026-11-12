import { describe, expect, it } from 'vitest'
import { emphasisSegments } from '../src/emphasis'

describe('emphasisSegments', () => {
  it('turns paired double asterisks into bold segments', () => {
    expect(emphasisSegments('**reach out** means **contact** someone')).toEqual([
      { text: 'reach out', bold: true },
      { text: ' means ', bold: false },
      { text: 'contact', bold: true },
      { text: ' someone', bold: false },
    ])
  })

  it('keeps text without markers or with an unpaired marker literal', () => {
    expect(emphasisSegments('plain help')).toEqual([{ text: 'plain help', bold: false }])
    expect(emphasisSegments('a ** b')).toEqual([{ text: 'a ** b', bold: false }])
  })

  it('never produces markup from the text itself', () => {
    expect(emphasisSegments('**<img src=x onerror=alert(1)>**')).toEqual([
      { text: '<img src=x onerror=alert(1)>', bold: true },
    ])
  })
})
