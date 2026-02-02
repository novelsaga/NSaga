/**
 * RPC Handler - 处理 JSON-RPC 请求和响应
 *
 * 此模块完全独立于运行时环境，可在 Node.js/Bun/Deno 中共享使用
 */

import type { Service, ServiceMethod } from './interfaces/service'
import type { RPCErrorObject, RPCId, RPCNotification, RPCRequest, RPCResponse } from './types/rpc'

import { isRPCNotification, isRPCRequest, RPCErrorCode } from './types/rpc'

export type { Service, ServiceMethod }

export class RPCHandler {
  private services = new Map<string, Service>()

  registerService(name: string, service: Service): void {
    this.services.set(name, service)
  }

  unregisterService(name: string): boolean {
    return this.services.delete(name)
  }

  async handleMessage(message: unknown): Promise<RPCResponse | null> {
    if (!isRPCRequest(message)) {
      return this.createErrorResponse(null, RPCErrorCode.InvalidRequest, 'Invalid Request')
    }

    if (isRPCNotification(message)) {
      await this.handleNotification(message)
      return null
    }

    return await this.handleRequest(message)
  }

  private async handleRequest(request: RPCRequest): Promise<RPCResponse> {
    try {
      const [serviceName, methodName] = this.parseMethod(request.method)

      const service = this.services.get(serviceName)
      if (!service) {
        return this.createErrorResponse(
          request.id ?? null,
          RPCErrorCode.ServiceNotFound,
          `Service not found: ${serviceName}`,
        )
      }

      const method: ServiceMethod | undefined = service[methodName]
      if (typeof method !== 'function') {
        return this.createErrorResponse(
          request.id ?? null,
          RPCErrorCode.MethodNotFound,
          `Method not found: ${request.method}`,
        )
      }

      const result: unknown = await Promise.resolve(method.call(service, request.params))

      return {
        jsonrpc: '2.0',
        result,
        id: request.id ?? null,
      }
    } catch (error) {
      return this.createErrorResponse(
        request.id ?? null,
        RPCErrorCode.InternalError,
        error instanceof Error ? error.message : 'Internal error',
        error,
      )
    }
  }

  private async handleNotification(notification: RPCNotification): Promise<void> {
    try {
      const [serviceName, methodName] = this.parseMethod(notification.method)
      const service = this.services.get(serviceName)

      if (service) {
        const method: ServiceMethod | undefined = service[methodName]
        if (typeof method === 'function') {
          await Promise.resolve(method.call(service, notification.params))
        }
      }
    } catch (error) {
      console.error('Error handling notification:', error)
    }
  }

  private parseMethod(method: string): [string, string] {
    const parts = method.split('.')
    if (parts.length !== 2) {
      throw new Error(`Invalid method format: ${method}. Expected "service.method"`)
    }
    return [parts[0], parts[1]]
  }

  private createErrorResponse(id: RPCId, code: number, message: string, data?: unknown): RPCResponse {
    const error: RPCErrorObject = { code, message }
    if (data !== undefined) {
      error.data = data
    }

    return {
      jsonrpc: '2.0',
      error,
      id,
    }
  }
}
