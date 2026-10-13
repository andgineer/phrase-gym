import { expect, it } from 'vitest'
import { mergeSupportFacts } from '../src/supportState'
import type { PracticeSession } from '../src/practiceTypes'

for (const sequence of [1, 2])
  it(`merges independent facts without changing originals when retained sequence is ${sequence}`, () => {
    const current = {
      last_sequence: sequence,
      support_exposures: { a: ['2026-10-13T00:01:00Z'] },
      attempts: [{ original: { supported: false } }],
    } as unknown as PracticeSession
    const incoming = {
      last_sequence: 1,
      support_exposures: { a: ['2026-10-13T00:00:00Z'], b: ['2026-10-13T00:02:00Z'] },
    } as unknown as PracticeSession
    expect(mergeSupportFacts(current, incoming)).toEqual({
      a: ['2026-10-13T00:00:00Z', '2026-10-13T00:01:00Z'],
      b: ['2026-10-13T00:02:00Z'],
    })
    expect(current.support_exposures?.a).toEqual(['2026-10-13T00:01:00Z'])
    expect(current.attempts[0]?.original?.supported).toBe(false)
  })
