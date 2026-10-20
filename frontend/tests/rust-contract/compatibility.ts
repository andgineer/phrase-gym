import type { paths as PythonPaths } from '../../src/api.generated'
import type { paths as RustPaths } from './api.generated'

// Mapping flattens aliases and intersections so only property names, optionality and types are compared.
// English library payload and reference-only readiness are intentional Rust-only additions.
type Normalized<T> = T extends 'study' | 'next'
  ? never
  : T extends { content: unknown; retrieval_ready: boolean }
    ? { [Key in Exclude<keyof T, 'english'>]: Normalized<T[Key]> }
    : T extends { due_count: number; reference_only_count: number }
      ? { [Key in Exclude<keyof T, 'reference_only_count'>]: Normalized<T[Key]> }
      : T extends object
        ? { [Key in keyof T]: Normalized<T[Key]> }
        : T
type Exact<A, B> =
  (<T>() => T extends Normalized<A> ? 1 : 2) extends <T>() => T extends Normalized<B> ? 1 : 2
    ? true
    : false
type JsonRequest<Operation> = Operation extends {
  requestBody: { content: { 'application/json': infer Body } }
}
  ? Body
  : never
type JsonSuccess<Operation> = Operation extends {
  responses: { 200: { content: { 'application/json': infer Body } } }
}
  ? Body
  : never
type Parameters<Operation> = Operation extends { parameters: infer Accepted }
  ? Pick<Accepted, Extract<keyof Accepted, 'query' | 'path'>>
  : never

type Same<Python, Rust> = [
  Exact<JsonRequest<Rust>, JsonRequest<Python>>,
  Exact<JsonSuccess<Rust>, JsonSuccess<Python>>,
  Exact<Parameters<Rust>, Parameters<Python>>,
]

// Each Rust endpoint must accept and return exactly what the retained client already sends and reads.
export const exposure: Same<
  PythonPaths['/api/v1/exposures']['post'],
  RustPaths['/api/v1/exposures']['post']
> = [true, true, true]
export const lessons: Same<
  PythonPaths['/api/v1/lessons']['get'],
  RustPaths['/api/v1/lessons']['get']
> = [true, true, true]
export const phraseSearch: Same<
  PythonPaths['/api/v1/phrases/search']['get'],
  RustPaths['/api/v1/phrases/search']['get']
> = [true, true, true]
export const lessonPhrases: Same<
  PythonPaths['/api/v1/lessons/{lesson_id}/phrases']['get'],
  RustPaths['/api/v1/lessons/{lesson_id}/phrases']['get']
> = [true, true, true]
export const phrase: Same<
  PythonPaths['/api/v1/phrases/{target_id}']['get'],
  RustPaths['/api/v1/phrases/{target_id}']['get']
> = [true, true, true]
export const dialogues: Same<
  PythonPaths['/api/v1/dialogues']['get'],
  RustPaths['/api/v1/dialogues']['get']
> = [true, true, true]
export const dialogue: Same<
  PythonPaths['/api/v1/dialogues/{dialogue_id}']['get'],
  RustPaths['/api/v1/dialogues/{dialogue_id}']['get']
> = [true, true, true]
export const readiness: Same<
  PythonPaths['/api/v1/sessions/readiness']['get'],
  RustPaths['/api/v1/sessions/readiness']['get']
> = [true, true, true]
export const mediaUpload: Same<
  PythonPaths['/api/v1/media']['post'],
  RustPaths['/api/v1/media']['post']
> = [true, true, true]
export const mediaMetadata: Same<
  PythonPaths['/api/v1/media/{media_id}/metadata']['get'],
  RustPaths['/api/v1/media/{media_id}/metadata']['get']
> = [true, true, true]
export const mediaReconciliation: Same<
  PythonPaths['/api/v1/media/reconcile']['post'],
  RustPaths['/api/v1/media/reconcile']['post']
> = [true, true, true]
export const settings: Same<
  PythonPaths['/api/v1/settings']['get'],
  RustPaths['/api/v1/settings']['get']
> = [true, true, true]
export const settingsReplacement: Same<
  PythonPaths['/api/v1/settings']['put'],
  RustPaths['/api/v1/settings']['put']
> = [true, true, true]
export const sessionStart: Same<
  PythonPaths['/api/v1/sessions']['post'],
  RustPaths['/api/v1/sessions']['post']
> = [true, true, true]
export const currentSession: Same<
  PythonPaths['/api/v1/sessions/current']['get'],
  RustPaths['/api/v1/sessions/current']['get']
> = [true, true, true]
export const session: Same<
  PythonPaths['/api/v1/sessions/{session_id}']['get'],
  RustPaths['/api/v1/sessions/{session_id}']['get']
> = [true, true, true]
export const sessionTransition: Same<
  PythonPaths['/api/v1/sessions/{session_id}']['patch'],
  RustPaths['/api/v1/sessions/{session_id}']['patch']
> = [true, true, true]
export const practiceEvents: Same<
  PythonPaths['/api/v1/sessions/{session_id}/events']['post'],
  RustPaths['/api/v1/sessions/{session_id}/events']['post']
> = [true, true, true]
export const practiceReceipts: Same<
  PythonPaths['/api/v1/sessions/{session_id}/events']['get'],
  RustPaths['/api/v1/sessions/{session_id}/events']['get']
> = [true, true, true]
export const takeUpload: Same<
  PythonPaths['/api/v1/recordings']['post'],
  RustPaths['/api/v1/recordings']['post']
> = [true, true, true]
export const phraseTakes: Same<
  PythonPaths['/api/v1/phrases/{target_id}/recordings']['get'],
  RustPaths['/api/v1/phrases/{target_id}/recordings']['get']
> = [true, true, true]
// Reference audio and takes are fetched by URL; only their address and parameters are contracts.
export const mediaBytes: Exact<
  Parameters<RustPaths['/api/v1/media/{media_id}']['get']>,
  Parameters<PythonPaths['/api/v1/media/{media_id}']['get']>
> = true
export const takeBytes: Exact<
  Parameters<RustPaths['/api/v1/recordings/{recording_id}']['get']>,
  Parameters<PythonPaths['/api/v1/recordings/{recording_id}']['get']>
> = true

// The comparison must reject added optional fields on either side and in nested objects.
type PythonExposure = PythonPaths['/api/v1/exposures']['post']
type RustExposure = RustPaths['/api/v1/exposures']['post']
// @ts-expect-error an extra optional request field on the Rust side
export const rustExtraField: Exact<
  JsonRequest<PythonExposure> & { source?: string },
  JsonRequest<PythonExposure>
> = true
// @ts-expect-error an extra optional response field on the Python side
export const pythonExtraField: Exact<
  JsonSuccess<RustExposure>,
  JsonSuccess<PythonExposure> & { source?: string }
> = true
// @ts-expect-error a nested optional field
export const nestedExtraField: Exact<
  { receipt: { id: string } },
  { receipt: { id: string; at?: string } }
> = true
// @ts-expect-error a required field that became optional
export const relaxedField: Exact<{ id?: string }, { id: string }> = true
// @ts-expect-error a query parameter that became nullable
export const nullableParameter: Exact<
  { query: { revision?: number | null } },
  { query: { revision?: number } }
> = true
