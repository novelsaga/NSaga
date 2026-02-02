/**
 * Transport 层 - 处理 stdin/stdout 通信（Deno 原生 API）
 */

// eslint-disable-next-line @typescript-eslint/triple-slash-reference
/// <reference path="../deno.d.ts" />

import type { MessageHandler, Transport } from '@nsaga/bridge-core/interfaces/transport'
import type { RPCResponse } from '@nsaga/bridge-core/types/rpc'

export class StdioTransport implements Transport {
  private messageHandlers: Set<MessageHandler> = new Set()
  private decoder = new TextDecoder()
  private encoder = new TextEncoder()
  private buffer = ''

  constructor() {
    void this.startReading()
  }

  /**
   * 启动读取 stdin
   */
  private async startReading(): Promise<void> {
    try {
      // Deno.stdin.readable 是一个 ReadableStream
      const reader = Deno.stdin.readable.getReader()

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
      Deno.exit(1)
    }
  }

  /**
   * 发送消息到 stdout
   */
  send(message: RPCResponse): void {
    const json = JSON.stringify(message)
    const writer = Deno.stdout.writable.getWriter()
    const encoded = this.encoder.encode(json + '\n')
    void writer.write(encoded).then(() => writer.releaseLock())
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
    Deno.exit(0)
  }

  /**
   * 关闭 transport
   */
  close(): void {
    // Deno 的 stdin 会在进程结束时自动关闭
    Deno.exit(0)
  }
}
