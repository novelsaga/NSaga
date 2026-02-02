#!/usr/bin/env node
import { dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

import { buildBridge } from '@nsaga/build-tool'

const __dirname = dirname(fileURLToPath(import.meta.url))

void buildBridge({
  name: 'bridge-nodejs',
  rootDir: __dirname,
  autoScanSrc: true,
})
