/**
 * Transport 层 - 处理 stdin/stdout 通信
 */

import type { Interface as ReadlineInterface } from 'node:readline'

import type { MessageHandler, Transport } from '@nsaga/bridge-core/interfaces/transport'
import type { RPCResponse } from '@nsaga/bridge-core/types/rpc'

import { createInterface } from 'node:readline'

export class StdioTransport implements Transport {
  private readline: ReadlineInterface
  private messageHandlers: Set<MessageHandler> = new Set()

  constructor() {
    // 创建 readline 接口读取 stdin
    this.readline = createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: false,
    })

    // 监听每一行输入
    this.readline.on('line', (line) => {
      const trimmed = line.trim()
      if (trimmed) {
        this.handleMessage(trimmed)
      }
    })

    // 监听 stdin 关闭
    this.readline.on('close', () => {
      this.onClose()
    })
  }

  /**
   * 发送消息到 stdout
   */
  send(message: RPCResponse): void {
    const json = JSON.stringify(message)
    // 使用 console.log 输出到 stdout（不会被 esbuild 或其他工具干扰）
    console.log(json)
  }

  /**
   * 注册消息处理器
   */
  onMessage(handler: MessageHandler): void {
    this.messageHandlers.add(handler)
  }

  /**
   * 处理接收到的消息
   */
  private handleMessage(message: string): void {
    for (const handler of this.messageHandlers) {
      try {
        void handler(message)
      } catch (error) {
        console.error('Error in message handler:', error)
      }
    }
  }

  /**
   * 连接关闭时的处理
   */
  private onClose(): void {
    // stdin 关闭，退出进程
    process.exit(0)
  }

  /**
   * 关闭 transport
   */
  close(): void {
    this.readline.close()
  }
}
