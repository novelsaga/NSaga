/// <reference types="node" />

import * as esbuild from "esbuild";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { readdir, mkdir } from "node:fs/promises";
import { exec } from "node:child_process";
import { promisify } from "node:util";

const execAsync = promisify(exec);
const __dirname = dirname(fileURLToPath(import.meta.url));

async function build() {
  const startTime = Date.now();

  try {
    // 获取所有 TypeScript 文件
    const typesFiles = await readdir(resolve(__dirname, "src/types"));
    const interfacesFiles = await readdir(resolve(__dirname, "src/interfaces"));

    const entryPoints = [
      ...typesFiles
        .filter((f) => f.endsWith(".ts"))
        .map((f) => resolve(__dirname, "src/types", f)),
      ...interfacesFiles
        .filter((f) => f.endsWith(".ts"))
        .map((f) => resolve(__dirname, "src/interfaces", f)),
    ];

    // 确保输出目录存在
    await mkdir(resolve(__dirname, "dist"), { recursive: true });

    // 1. 使用 esbuild 编译 JS
    await esbuild.build({
      entryPoints,
      bundle: false, // 不打包，保持模块结构
      outdir: "dist",
      outbase: "src",
      platform: "node",
      target: "node20",
      format: "esm",
      sourcemap: false,
      minify: false,
      logLevel: "info",
      loader: {
        ".ts": "ts",
      },
    });

    // 2. 使用 tsc 生成 .d.ts 文件
    console.log("Generating .d.ts files...");
    await execAsync("npx tsc --emitDeclarationOnly");

    const duration = Date.now() - startTime;
    console.log(`✅ Build successful! Time: ${duration}ms`);
    console.log(`📦 Output: dist/`);
  } catch (error) {
    console.error("❌ Build failed:", error);
    process.exit(1);
  }
}

build();
