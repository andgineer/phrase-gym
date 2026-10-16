import type { paths as PythonPaths } from '../../src/api.generated'
import type { paths as RustPaths } from './api.generated'

// Mapping flattens aliases and intersections so only property names, optionality and types are compared.
type Normalized<T> = T extends object ? { [Key in keyof T]: Normalized<T[Key]> } : T
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

type PythonExposure = PythonPaths['/api/v1/exposures']['post']
type RustExposure = RustPaths['/api/v1/exposures']['post']

// Each Rust endpoint must accept and return exactly what the retained client already sends and reads.
export const exposureRequest: Exact<JsonRequest<RustExposure>, JsonRequest<PythonExposure>> = true
export const exposureReceipt: Exact<JsonSuccess<RustExposure>, JsonSuccess<PythonExposure>> = true

// The comparison must reject added optional fields on either side and in nested objects.
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
