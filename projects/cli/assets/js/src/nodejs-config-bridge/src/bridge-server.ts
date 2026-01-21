/**
 * Node.js 服务 - 供 Rust 调用
 *
 * 通过 stdin/stdout 进行 JSON-RPC 通信
 */

import readline from "node:readline";
import { NovelSagaConfig } from "./types";

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false,
});

const GLOBAL_SETTINGS = {
  CONFIG_IS_COMMONJS: process.env.NSAGA_CLI_CONFIG_IS_COMMONJS === "true",
  CONFIG_IS_TYPESCRIPT: process.env.NSAGA_CLI_CONFIG_IS_TYPESCRIPT === "true",
  CONFIG_PATH: process.env.NSAGA_CLI_CONFIG_PATH,
  PROJECT_ROOT: process.env.NSAGA_CLI_PROJECT_ROOT,
  IS_DEV: process.env.NSAGA_CLI_IS_DEV === "true",
  VERBOSE_LOG: process.env.NSAGA_CLI_VERBOSE_LOG === "true",
};

rl.on("line", async (line) => {
  let request;
  try {
    request = JSON.parse(line);
    let result;
    switch (request.method) {
      // 获取配置文件基本参数
      case "get_config": {
        if (GLOBAL_SETTINGS.CONFIG_PATH === undefined) {
          throw new Error("CONFIG_PATH 未定义");
        }
        const configModule = await import(GLOBAL_SETTINGS.CONFIG_PATH);
        let configMain:
          | NovelSagaConfig
          | ((globalSettings: typeof GLOBAL_SETTINGS) => NovelSagaConfig);
        if (GLOBAL_SETTINGS.CONFIG_IS_COMMONJS) {
          configMain = configModule;
        } else {
          configMain = configModule.default;
        }
        if (typeof configMain === "function") {
          result = configMain(GLOBAL_SETTINGS);
        } else {
          result = configMain;
        }
        // 过滤掉所有function，确保可以序列化
        result = JSON.parse(
          JSON.stringify(result, (_, value) =>
            typeof value === "function" ? undefined : value
          )
        );
        break;
      }
      default:
        throw new Error(`Unknown method: ${request.method}`);
    }
    console.log(JSON.stringify({ id: request.id, result }));
  } catch (error) {
    console.error(
      JSON.stringify({
        id: request.id || 0,
        error: (error as Error).message,
        stack: (error as Error).stack,
      })
    );
  }
});

rl.on("close", () => {
  console.error("\n👋 Node.js 服务已关闭");
  process.exit(0);
});

// 优雅退出
process.on("SIGTERM", () => {
  console.error("\n📛 收到 SIGTERM 信号");
  process.exit(0);
});

process.on("SIGINT", () => {
  console.error("\n📛 收到 SIGINT 信号");
  process.exit(0);
});
