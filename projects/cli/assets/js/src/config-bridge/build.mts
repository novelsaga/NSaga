/// <reference types="node" />

import * as esbuild from "esbuild";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdir, writeFile } from "node:fs/promises";

const __dirname = dirname(fileURLToPath(import.meta.url));

async function build() {
  const startTime = Date.now();

  try {
    // 统一输出目录: projects/cli/assets/js/dist/
    const distDir = resolve(__dirname, "../../dist");
    await mkdir(distDir, { recursive: true });

    await esbuild.build({
      entryPoints: [resolve(__dirname, "src/index.ts")],
      bundle: true,
      outfile: resolve(distDir, "config-bridge.js"),
      platform: "node",
      target: "node20",
      format: "esm",
      sourcemap: false,
      minify: false,
      external: ["node:*"],
      treeShaking: true,
      logLevel: "info",
      loader: {
        ".ts": "ts",
      },
    });

    // 在 dist 目录创建 package.json（标记为 ESM）
    await writeFile(
      resolve(distDir, "package.json"),
      JSON.stringify({ type: "module" }, null, 2)
    );

    const duration = Date.now() - startTime;
    console.log(`✅ Build successful! Time: ${duration}ms`);
    console.log(`📦 Output: ../../dist/config-bridge.js`);
  } catch (error) {
    console.error("❌ Build failed:", error);
    process.exit(1);
  }
}

build();
