/**
 * Bridge Server - 整合 Transport 和 RPC Handler
 */

import { StdioTransport } from "./transport.js";
import { RPCHandler, type Service } from "./rpc-handler.js";
import { RPCErrorCode } from "@nsaga/bridge-core/types/rpc";

export interface BridgeServerConfig {
  name: string;
  version: string;
}

export class BridgeServer {
  private transport: StdioTransport;
  private rpcHandler: RPCHandler;
  private config: BridgeServerConfig;

  constructor(config: BridgeServerConfig) {
    this.config = config;
    this.transport = new StdioTransport();
    this.rpcHandler = new RPCHandler();
  }

  /**
   * 注册服务
   */
  registerService(name: string, service: Service): void {
    this.rpcHandler.registerService(name, service);
  }

  /**
   * 启动服务器
   */
  start(): void {
    // 输出启动信息到 stderr（不会干扰 stdout 的 JSON-RPC 通信）
    console.error(`[${this.config.name}] v${this.config.version} started`);

    // 监听消息
    this.transport.onMessage(async (message) => {
      try {
        // 解析 JSON
        const parsed = JSON.parse(message);

        // 处理 RPC 请求
        const response = await this.rpcHandler.handleMessage(parsed);

        // 如果有响应（非通知），发送回去
        if (response) {
          this.transport.send(response);
        }
      } catch (error) {
        // JSON 解析错误
        this.transport.send({
          jsonrpc: "2.0",
          error: {
            code: RPCErrorCode.ParseError,
            message: "Parse error",
            data: error instanceof Error ? error.message : String(error),
          },
          id: null,
        });
      }
    });

    // 捕获未处理的异常
    process.on("uncaughtException", (error) => {
      console.error(`[${this.config.name}] Uncaught exception:`, error);
      process.exit(1);
    });

    process.on("unhandledRejection", (reason) => {
      console.error(`[${this.config.name}] Unhandled rejection:`, reason);
      process.exit(1);
    });
  }

  /**
   * 停止服务器
   */
  stop(): void {
    this.transport.close();
  }
}
