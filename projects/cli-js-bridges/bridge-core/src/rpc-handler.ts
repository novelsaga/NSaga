/**
 * RPC Handler - 处理 JSON-RPC 请求和响应
 *
 * 此模块完全独立于运行时环境，可在 Node.js/Bun/Deno 中共享使用
 */

import type { Service } from './interfaces/service'
import type { RPCErrorObject, RPCId, RPCNotification, RPCRequest, RPCResponse } from './types/rpc'

import { isRPCNotification, isRPCRequest, RPCErrorCode } from './types/rpc'

/** 服务方法处理器 */
export type ServiceMethod = (params: unknown) => Promise<unknown> | unknown

export { Service }

export class RPCHandler {
  private services = new Map<string, Service>()

  /**
   * 注册服务
   */
  registerService(name: string, service: Service): void {
    this.services.set(name, service)
  }

  /**
   * 取消注册服务
   */
  unregisterService(name: string): boolean {
    return this.services.delete(name)
  }

  /**
   * 处理 RPC 消息
   */
  async handleMessage(message: unknown): Promise<RPCResponse | null> {
    // 验证消息格式
    if (!isRPCRequest(message)) {
      return this.createErrorResponse(null, RPCErrorCode.InvalidRequest, 'Invalid Request')
    }

    // 通知不需要响应
    if (isRPCNotification(message)) {
      await this.handleNotification(message)
      return null
    }

    // 处理请求
    return await this.handleRequest(message)
  }

  /**
   * 处理请求
   */
  private async handleRequest(request: RPCRequest): Promise<RPCResponse> {
    try {
      // 解析方法名：格式为 "service.method"
      const [serviceName, methodName] = this.parseMethod(request.method)

      // 查找服务
      const service = this.services.get(serviceName)
      if (!service) {
        return this.createErrorResponse(request.id!, RPCErrorCode.ServiceNotFound, `Service not found: ${serviceName}`)
      }

      // 查找方法
      const method = service[methodName]
      if (typeof method !== 'function') {
        return this.createErrorResponse(request.id!, RPCErrorCode.MethodNotFound, `Method not found: ${request.method}`)
      }

      // 执行方法
      const result = await Promise.resolve(method.call(service, request.params))

      // 返回成功响应
      return {
        jsonrpc: '2.0',
        result,
        id: request.id!,
      }
    } catch (error) {
      // 返回错误响应
      return this.createErrorResponse(
        request.id!,
        RPCErrorCode.InternalError,
        error instanceof Error ? error.message : 'Internal error',
        error,
      )
    }
  }

  /**
   * 处理通知
   */
  private async handleNotification(notification: RPCNotification): Promise<void> {
    try {
      const [serviceName, methodName] = this.parseMethod(notification.method)
      const service = this.services.get(serviceName)

      if (service) {
        const method = service[methodName]
        if (typeof method === 'function') {
          await Promise.resolve(method.call(service, notification.params))
        }
      }
    } catch (error) {
      // 通知不返回错误，只记录
      console.error('Error handling notification:', error)
    }
  }

  /**
   * 解析方法名
   */
  private parseMethod(method: string): [string, string] {
    const parts = method.split('.')
    if (parts.length !== 2) {
      throw new Error(`Invalid method format: ${method}. Expected "service.method"`)
    }
    return [parts[0], parts[1]]
  }

  /**
   * 创建错误响应
   */
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
