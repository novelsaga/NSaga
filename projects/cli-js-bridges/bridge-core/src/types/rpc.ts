/**
 * JSON-RPC 2.0 协议类型定义
 * @see https://www.jsonrpc.org/specification
 */

/** JSON-RPC 请求 ID 类型 */
export type RPCId = string | number | null

/** JSON-RPC 请求 */
export interface RPCRequest {
  jsonrpc: '2.0'
  method: string
  params?: unknown
  id?: RPCId
}

/** JSON-RPC 响应（成功） */
export interface RPCSuccessResponse {
  jsonrpc: '2.0'
  result: unknown
  id: RPCId
}

/** JSON-RPC 错误对象 */
export interface RPCErrorObject {
  code: number
  message: string
  data?: unknown
}

/** JSON-RPC 响应（错误） */
export interface RPCErrorResponse {
  jsonrpc: '2.0'
  error: RPCErrorObject
  id: RPCId
}

/** JSON-RPC 响应 */
export type RPCResponse = RPCSuccessResponse | RPCErrorResponse

/** JSON-RPC 通知（无 id） */
export interface RPCNotification {
  jsonrpc: '2.0'
  method: string
  params?: unknown
}

/** JSON-RPC 标准错误码 */
export enum RPCErrorCode {
  ParseError = -32700,
  InvalidRequest = -32600,
  MethodNotFound = -32601,
  InvalidParams = -32602,
  InternalError = -32603,
  // 自定义错误码范围: -32000 到 -32099
  ServiceNotFound = -32001,
  ServiceError = -32002,
  TimeoutError = -32003,
}

/** 类型守卫：检查是否为请求 */
export function isRPCRequest(message: unknown): message is RPCRequest {
  return (
    typeof message === 'object' &&
    message !== null &&
    'jsonrpc' in message &&
    message.jsonrpc === '2.0' &&
    'method' in message &&
    typeof message.method === 'string'
  )
}

/** 类型守卫：检查是否为通知 */
export function isRPCNotification(message: unknown): message is RPCNotification {
  return isRPCRequest(message) && !('id' in message)
}

/** 类型守卫：检查是否为错误响应 */
export function isRPCErrorResponse(response: RPCResponse): response is RPCErrorResponse {
  return 'error' in response
}
