#!/usr/bin/env node
/// <reference types="node" />
import { exec } from 'node:child_process'
import { dirname } from 'node:path'
import { fileURLToPath } from 'node:url'
import { promisify } from 'node:util'

import { buildBridge } from '@nsaga/build-tool'

const execAsync = promisify(exec)
const __dirname = dirname(fileURLToPath(import.meta.url))

await buildBridge({
  name: 'bridge-deno',
  rootDir: __dirname,
  entryPoints: ['src/transport.ts', 'src/index.ts'],
  preBuild: async () => {
    // 生成 deno.d.ts
    try {
      console.log('📝 Generating Deno type definitions...')
      await execAsync('deno types > deno.d.ts', { cwd: __dirname })
      console.log('✅ deno.d.ts generated')
    } catch (_err) {
      console.warn('⚠️  Failed to generate deno.d.ts, using existing file if available')
      console.warn("   Make sure 'deno' is in PATH, or manually generate with: deno types > deno.d.ts")
    }
  },
})
