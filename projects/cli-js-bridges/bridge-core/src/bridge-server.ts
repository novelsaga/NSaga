/**
 * Bridge Server - 整合 Transport 和 RPC Handler
 *
 * 使用依赖注入模式，支持不同运行时环境的错误处理
 */

import type { Service } from './interfaces/service'
import type { Transport } from './interfaces/transport'

import { RPCHandler } from './rpc-handler'
import { RPCErrorCode } from './types/rpc'

export interface BridgeServerConfig {
  name: string
  version: string
}

/**
 * 错误处理器接口 - 由各运行时实现
 */
export interface ErrorHandler {
  /**
   * 设置错误处理钩子
   * @param onError 错误回调函数
   * @param onExit 退出回调函数
   */
  setupErrorHandlers(
    onError: (error: Error, type: 'uncaught' | 'unhandled') => void,
    onExit: (code: number) => void,
  ): void
}

export class BridgeServer {
  private transport: Transport
  private rpcHandler: RPCHandler
  private config: BridgeServerConfig
  private errorHandler?: ErrorHandler

  constructor(config: BridgeServerConfig, transport: Transport, errorHandler?: ErrorHandler) {
    this.config = config
    this.transport = transport
    this.rpcHandler = new RPCHandler()
    this.errorHandler = errorHandler
  }

  /**
   * 注册服务
   */
  registerService(name: string, service: Service): void {
    this.rpcHandler.registerService(name, service)
  }

  /**
   * 启动服务器
   */
  start(): void {
    // 输出启动信息到 stderr（不会干扰 stdout 的 JSON-RPC 通信）
    console.error(`[${this.config.name}] v${this.config.version} started`)

    // 监听消息
    this.transport.onMessage(async (message) => {
      try {
        // 解析 JSON
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment --- IGNORE ---
        const parsed = JSON.parse(message)

        // 处理 RPC 请求
        const response = await this.rpcHandler.handleMessage(parsed)

        // 如果有响应（非通知），发送回去
        if (response) {
          this.transport.send(response)
        }
      } catch (error) {
        // JSON 解析错误
        this.transport.send({
          jsonrpc: '2.0',
          error: {
            code: RPCErrorCode.ParseError,
            message: 'Parse error',
            data: error instanceof Error ? error.message : String(error),
          },
          id: null,
        })
      }
    })

    // 设置错误处理
    if (this.errorHandler) {
      this.errorHandler.setupErrorHandlers(
        (error, type) => {
          console.error(
            `[${this.config.name}] ${type === 'uncaught' ? 'Uncaught exception' : 'Unhandled rejection'}:`,
            error,
          )
        },
        (_code) => {
          // 退出逻辑由各运行时实现
        },
      )
    }
  }

  /**
   * 停止服务器
   */
  stop(): void {
    this.transport.close()
  }
}
