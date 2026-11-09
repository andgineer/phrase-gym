const MONTH_DAYS = 365 / 12
const YEAR_DAYS = 365

const tenths = (value: number) => String(Math.round(value * 10) / 10)

export function returnSpan(days: number): string {
  if (days < MONTH_DAYS) return `${days}d`
  if (days < YEAR_DAYS) return `${tenths(days / MONTH_DAYS)}m`
  return `${tenths(days / YEAR_DAYS)}y`
}

export const returnDescription = (days: number) =>
  days === 1 ? 'Back tomorrow' : `Back in ${days} days`
