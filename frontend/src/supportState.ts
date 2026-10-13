import type { PracticeSession } from './practiceTypes'

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
