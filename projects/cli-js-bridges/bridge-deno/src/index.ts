import type { BridgeServerConfig, ErrorHandler } from '@nsaga/bridge-core/bridge-server'

import { BridgeServer } from '@nsaga/bridge-core/bridge-server'

import { StdioTransport } from './transport.js'

/**
 * Deno Bridge Adapter - 使用 Deno 原生 API
 */

/// <reference path="../deno.d.ts" />

/**
 * Deno 错误处理器
 */
class DenoErrorHandler implements ErrorHandler {
  setupErrorHandlers(
    onError: (error: Error, type: 'uncaught' | 'unhandled') => void,
    onExit: (code: number) => void,
  ): void {
    // Deno 使用全局事件监听器
    globalThis.addEventListener('error', (event) => {
      const error = event.error instanceof Error ? event.error : new Error(event.message)
      onError(error, 'uncaught')
      onExit(1)
    })

    globalThis.addEventListener('unhandledrejection', (event) => {
      const error = event.reason instanceof Error ? event.reason : new Error(String(event.reason))
      onError(error, 'unhandled')
      onExit(1)
    })
  }
}

/**
 * 创建 Deno Bridge Server 实例（工厂函数）
 */
export function createBridgeServer(config: BridgeServerConfig): BridgeServer {
  const transport = new StdioTransport()
  const errorHandler = new DenoErrorHandler()

  // 在 stderr 输出 runtime 标识
  console.error(`[${config.name}] Bridge server started (v${config.version}) on Deno`)

  return new BridgeServer(config, transport, errorHandler)
}

// 重新导出核心类型和类，方便使用
export { BridgeServer, type BridgeServerConfig } from '@nsaga/bridge-core/bridge-server'
export { RPCHandler, type Service } from '@nsaga/bridge-core/rpc-handler'
export { StdioTransport } from './transport.js'
