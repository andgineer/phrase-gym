import type { components } from './api.generated'
import type { components as RustComponents } from '../tests/rust-contract/api.generated'
import { ref } from 'vue'

export const serverUnavailable = ref(false)
export const connectionRetrying = ref(false)
export function requireServer() {
  if (serverUnavailable.value || connectionRetrying.value)
    throw new Error('Server connection required. Retry before continuing.')
}

export type Settings = components['schemas']['Settings']
export type Language = NonNullable<Settings['native_language']>
export type Phrase = RustComponents['schemas']['Phrase']
export type Lesson = components['schemas']['Lesson']
export type PhrasePage = RustComponents['schemas']['PhrasePage']
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
  let response: Response
  try {
    response = await fetch(`/api/v1${path}`, {
      ...options,
      cache: 'no-store',
      headers: { 'Content-Type': 'application/json', ...options?.headers },
    })
  } catch (cause) {
    serverUnavailable.value = true
    throw cause
  }
  if (!response.ok) {
    if (response.status >= 500) serverUnavailable.value = true
    const error = await response.json().catch(() => null)
    const detail = error?.detail
    throw new ApiError(
      typeof detail === 'string'
        ? detail
        : Array.isArray(detail)
          ? detail.map((issue) => `${issue.loc?.join('.')}: ${issue.msg}`).join('; ')
          : `Request failed (${response.status})`,
      response.status,
    )
  }
  try {
    return (await response.json()) as T
  } catch (cause) {
    serverUnavailable.value = true
    throw cause
  }
}

export function language(value: unknown): Language | null {
  return typeof value === 'string' && ['en', 'ru', 'sr', 'de'].includes(value)
    ? (value as Language)
    : null
}
