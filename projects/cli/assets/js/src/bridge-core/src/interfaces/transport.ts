/**
 * Transport 抽象接口 - 跨运行时的消息传输层
 */

import type { RPCResponse } from "../types/rpc.js";

/**
 * 消息处理器类型
 */
export type MessageHandler = (message: string) => void | Promise<void>;

/**
 * Transport 接口 - 定义消息传输的抽象
 */
export interface Transport {
  /**
   * 发送消息
   */
  send(message: RPCResponse): void;

  /**
   * 注册消息处理器
   */
  onMessage(handler: MessageHandler): void;

  /**
   * 关闭传输层
   */
  close(): void;
}
