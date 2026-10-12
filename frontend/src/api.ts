import type { components } from './api.generated'

export type Settings = components['schemas']['Settings']
export type Language = NonNullable<Settings['native_language']>
export type Phrase = components['schemas']['Phrase']
export type Lesson = components['schemas']['Lesson']
export type PhrasePage = components['schemas']['PhrasePage']
export type LessonPage = components['schemas']['LessonPage']
export type ExposureInput = components['schemas']['ExposureInput']

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message)
  }
}

export async function api<T>(path: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`/api/v1${path}`, {
    ...options,
    headers: { 'Content-Type': 'application/json', ...options?.headers },
  })
  if (!response.ok) {
    const error = await response.json().catch(() => null)
    throw new ApiError(
      typeof error?.detail === 'string' ? error.detail : `Request failed (${response.status})`,
      response.status,
    )
  }
  return response.json() as Promise<T>
}

export function language(value: unknown): Language | null {
  return typeof value === 'string' && ['en', 'ru', 'sr', 'de'].includes(value)
    ? (value as Language)
    : null
}
