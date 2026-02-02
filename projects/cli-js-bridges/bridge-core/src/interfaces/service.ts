/**
 * Service interface for RPC method dispatch
 */

export type ServiceMethod = (params: unknown) => unknown

export interface Service {
  [method: string]: ServiceMethod | undefined
}
