import type { PracticeSession } from './practiceTypes'

export function mergeSupportFacts(current: PracticeSession | undefined, incoming: PracticeSession) {
  const facts: Record<string, string[]> = {}
  for (const state of [current, incoming])
    for (const [target, times] of Object.entries(state?.support_exposures ?? {}))
      facts[target] = [...new Set([...(facts[target] ?? []), ...times])].sort()
  return facts
}

export function rememberSupport(state: PracticeSession, targetId: string, time: string) {
  const exposures = (state.support_exposures ??= {})
  const times = (exposures[targetId] ??= [])
  if (!times.includes(time)) times.push(time)
}
export function recentSupport(state: PracticeSession, targetId: string, time: string): boolean {
  return (state.support_exposures?.[targetId] ?? []).some((exposed) => {
    const elapsed = Date.parse(time) - Date.parse(exposed)
    return elapsed >= 0 && elapsed <= 300000
  })
}
