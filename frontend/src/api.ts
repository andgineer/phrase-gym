export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message)
  }
}

export async function api<T>(path: string, init?: RequestInit): Promise<T> {
  const headers = new Headers(init?.headers)
  if (init?.body) headers.set('content-type', 'application/json')
  const response = await fetch(`/api${path}`, { ...init, headers })
  if (!response.ok) {
    const body = (await response.json().catch(() => ({}))) as { detail?: unknown }
    const detail = Array.isArray(body.detail)
      ? body.detail
          .map((item) => {
            const error = item as { loc?: Array<string | number>; msg?: string }
            return `${error.loc?.slice(1).join('.') ?? 'package'}: ${error.msg ?? 'invalid'}`
          })
          .join('\n')
      : String(body.detail ?? `Request failed (${response.status})`)
    throw new ApiError(detail, response.status)
  }
  return response.json() as Promise<T>
}
