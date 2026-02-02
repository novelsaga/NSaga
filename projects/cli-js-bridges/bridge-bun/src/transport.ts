/**
 * Transport 层 - 处理 stdin/stdout 通信（Bun 原生 API）
 */

import type { MessageHandler, Transport } from '@nsaga/bridge-core/interfaces/transport'
import type { RPCResponse } from '@nsaga/bridge-core/types/rpc'

export class StdioTransport implements Transport {
  private messageHandlers: Set<MessageHandler> = new Set()
  private decoder = new TextDecoder()
  private buffer = ''

  constructor() {
    // 使用 Bun 原生 API 读取 stdin
    this.startReading()
  }

  /**
   * 启动读取 stdin
   */
  private async startReading(): Promise<void> {
    try {
      // Bun.stdin 是一个 ReadableStream
      const reader = Bun.stdin.stream().getReader()

      while (true) {
        const { done, value } = await reader.read()

        if (done) {
          this.onClose()
          break
        }

        // 解码并处理数据
        const text = this.decoder.decode(value, { stream: true })
        this.buffer += text

        // 按行处理消息
        const lines = this.buffer.split('\n')
        this.buffer = lines.pop() || ''

        for (const line of lines) {
          const trimmed = line.trim()
          if (trimmed) {
            this.handleMessage(trimmed)
          }
        }
      }
    } catch (error) {
      console.error('Error reading stdin:', error)
      process.exit(1)
    }
  }

  /**
   * 发送消息到 stdout
   */
  send(message: RPCResponse): void {
    const json = JSON.stringify(message)
    // 使用 Bun.write 直接写入 stdout
    Bun.write(Bun.stdout, json + '\n')
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
        handler(message)
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
    // Bun 的 stdin 会在进程结束时自动关闭
    process.exit(0)
  }
}
