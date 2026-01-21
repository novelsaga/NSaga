/**
 * Config Bridge - 配置文件加载服务
 */

import { BridgeServer } from "@nsaga/bridge-nodejs/bridge-server";
import { ConfigService } from "./services/config.js";

async function main() {
  // 创建 Bridge Server
  const server = new BridgeServer({
    name: "config-bridge",
    version: "1.0.0",
  });

  // 注册配置服务
  const configService = new ConfigService();
  server.registerService("config", configService);

  // 启动服务器
  server.start();
}

// 启动
main().catch((error) => {
  console.error("Fatal error:", error);
  process.exit(1);
});
