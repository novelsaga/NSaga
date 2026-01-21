import * as esbuild from "esbuild";
import { resolve } from "node:path";

async function build() {
  const startTime = Date.now();

  try {
    await esbuild.build({
      entryPoints: [resolve("src/bridge-server.ts")],
      bundle: true,
      outfile: "../../dist/nodejs-config-bridge/bridge-server.js",
      platform: "node",
      target: "node20",
      format: "esm",
      sourcemap: true,
      minify: false,
      // 外部化 Node.js 内置模块
      external: ["node:*"],
      // 生成可读的代码
      treeShaking: true,
      logLevel: "info",
      // TypeScript 支持
      loader: {
        ".ts": "ts",
      },
    });

    const duration = Date.now() - startTime;
    console.log(`✅ 构建成功! 耗时: ${duration}ms`);
    console.log(`📦 输出文件: dist/bridge-server.js`);
  } catch (error) {
    console.error("❌ 构建失败:", error);
    process.exit(1);
  }
}

build();
