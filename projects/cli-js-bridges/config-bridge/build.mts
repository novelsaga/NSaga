#!/usr/bin/env node
import { writeFile } from 'node:fs/promises'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { buildBridge } from '@nsaga/build-tool'

const __dirname = dirname(fileURLToPath(import.meta.url))
const distDir = resolve(__dirname, '../../cli/assets/js/dist')

await buildBridge({
  name: 'config-bridge',
  rootDir: __dirname,
  entryPoints: ['src/index.ts'],
  generateDts: false, // bundle 模式不需要 .d.ts
  esbuildOverrides: {
    bundle: true,
    outfile: resolve(distDir, 'config-bridge.js'),
    outdir: undefined,
    external: ['node:*'],
  },
  postBuild: async () => {
    // 在 dist 目录创建 package.json（标记为 ESM）
    await writeFile(resolve(distDir, 'package.json'), JSON.stringify({ type: 'module' }, null, 2))
    console.log(`📦 Output: ${distDir}/config-bridge.js`)
  },
})
