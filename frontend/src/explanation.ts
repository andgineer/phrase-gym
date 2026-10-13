import type { components } from './api.generated'

type Explanation = components['schemas']['Explanation']

export function preparedExplanation(value: Explanation | null | undefined): Explanation | null {
  if (!value) return null
  const text = [value.meaning, value.usage ?? '', ...(value.breakdown ?? [])]
  return text.some((part) => part.trim().length > 0) ? value : null
}
