import type { paths as PythonPaths } from '../../src/api.generated'
import type { paths as RustPaths } from './api.generated'

type Mutual<A, B> = [A] extends [B] ? ([B] extends [A] ? true : false) : false
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
export const exposureRequest: Mutual<JsonRequest<RustExposure>, JsonRequest<PythonExposure>> = true
export const exposureReceipt: Mutual<JsonSuccess<RustExposure>, JsonSuccess<PythonExposure>> = true
