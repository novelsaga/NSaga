import type { BridgeServerConfig, ErrorHandler } from '@nsaga/bridge-core/bridge-server'

import { BridgeServer } from '@nsaga/bridge-core/bridge-server'

import { StdioTransport } from './transport.js'

/**
 * Node.js Bridge Adapter - 使用 Node.js 原生 API
 */

/**
 * Node.js 错误处理器
 */
class NodeErrorHandler implements ErrorHandler {
  setupErrorHandlers(
    onError: (error: Error, type: 'uncaught' | 'unhandled') => void,
    onExit: (code: number) => void,
  ): void {
    process.on('uncaughtException', (error) => {
      onError(error, 'uncaught')
      onExit(1)
    })

    process.on('unhandledRejection', (reason) => {
      const error = reason instanceof Error ? reason : new Error(String(reason))
      onError(error, 'unhandled')
      onExit(1)
    })
  }
}

/**
 * 创建 Node.js Bridge Server 实例（工厂函数）
 */
export function createBridgeServer(config: BridgeServerConfig): BridgeServer {
  const transport = new StdioTransport()
  const errorHandler = new NodeErrorHandler()
  return new BridgeServer(config, transport, errorHandler)
}

// 重新导出核心类型和类，方便使用
export { BridgeServer, type BridgeServerConfig } from '@nsaga/bridge-core/bridge-server'
export { RPCHandler, type Service } from '@nsaga/bridge-core/rpc-handler'
export { StdioTransport } from './transport.js'
