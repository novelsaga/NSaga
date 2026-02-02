#!/usr/bin/env node
import { dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

import { buildBridge } from '@nsaga/build-tool'

const __dirname = dirname(fileURLToPath(import.meta.url))

await buildBridge({
  name: 'bridge-bun',
  rootDir: __dirname,
  entryPoints: ['src/transport.ts', 'src/index.ts'],
})
