# JS Bridges Module Guide

## Overview

TypeScript bridges provide JSON-RPC 2.0 services called by the Rust CLI over stdin/stdout.

## Structure

```
projects/cli-js-bridges/
├── build-tool/           # Shared build utilities (@nsaga/build-tool)
├── bridge-core/          # Runtime-agnostic core (@nsaga/bridge-core)
├── bridge-nodejs/        # Node.js adapter (@nsaga/bridge-nodejs)
├── bridge-bun/           # Bun adapter (@nsaga/bridge-bun)
├── bridge-deno/          # Deno adapter (@nsaga/bridge-deno)
└── config-bridge/        # Config loading service (@nsaga/config-bridge)
```

## Package Hierarchy

```
                    build-tool (dev dependency)
                         ↓
           ┌─────────────┴─────────────┐
           ↓                           ↓
      bridge-core              (all bridge-* packages)
           ↓
    ┌──────┼──────┐
    ↓      ↓      ↓
 nodejs   bun   deno  (runtime adapters)
    ↓      ↓      ↓
    └──────┼──────┘
           ↓
     config-bridge  (uses one adapter based on NSAGA_RUNTIME)
```

## Key Packages

### build-tool (`@nsaga/build-tool`)

Unified build system for all bridges:

```typescript
import { buildBridge } from '@nsaga/build-tool'

buildBridge({
  name: 'my-bridge',
  rootDir: __dirname,
  autoScanSrc: true, // Or specify entryPoints
  generateDts: true, // Generate .d.ts files
  preBuild: async () => {}, // Hook before build
  postBuild: async () => {}, // Hook after build
})
```

### bridge-core (`@nsaga/bridge-core`)

Runtime-agnostic shared code:

- `types/rpc.ts` - JSON-RPC 2.0 types
- `interfaces/transport.ts` - Abstract Transport interface
- `interfaces/service.ts` - Service interface
- `rpc-handler.ts` - RPC protocol handler
- `bridge-server.ts` - Bridge server (dependency injection)

### bridge-nodejs/bun/deno

Runtime-specific adapters (~120-150 lines each):

```typescript
// Each provides:
export function createBridgeServer(config: BridgeConfig): BridgeServer

// Usage:
const { createBridgeServer } = await import('@nsaga/bridge-nodejs')
const server = createBridgeServer({ name: 'my-bridge', version: '1.0.0' })
```

### config-bridge

Loads JS/TS config files and returns to Rust:

```typescript
// Environment: NSAGA_CLI_CONFIG_PATH, NSAGA_RUNTIME

// RPC Methods:
// - config.get: Load and return config from file path
```

## Creating a New Bridge

1. Create directory: `projects/cli-js-bridges/my-bridge/`

2. Create `package.json`:

   ```json
   {
     "name": "@nsaga/my-bridge",
     "type": "module",
     "devDependencies": {
       "@nsaga/build-tool": "workspace:*",
       "@nsaga/bridge-nodejs": "workspace:*"
     }
   }
   ```

3. Create `src/index.ts`:

   ```typescript
   import { createBridgeServer } from '@nsaga/bridge-nodejs'
   import { MyService } from './services/my-service.js'

   const server = createBridgeServer({
     name: 'my-bridge',
     version: '1.0.0',
   })
   server.registerService('myservice', new MyService())
   server.start()
   ```

4. Create `src/services/my-service.ts`:

   ```typescript
   import type { Service } from '@nsaga/bridge-core/interfaces/service'

   export class MyService implements Service {
     [method: string]: any

     async myMethod(params: { input: string }): Promise<{ output: string }> {
       return { output: params.input.toUpperCase() }
     }
   }
   ```

5. Create `build.mts`:

   ```typescript
   import { buildBridge } from '@nsaga/build-tool'
   buildBridge({
     name: 'my-bridge',
     rootDir: import.meta.dirname,
     entryPoints: ['src/index.ts'],
   })
   ```

6. Build: `./xtask.sh build-js`

7. Register in Rust `BridgeManager`

## JSON-RPC Communication

**Request format**:

```json
{"jsonrpc":"2.0","id":1,"method":"service.method","params":{...}}
```

**Response format**:

```json
{"jsonrpc":"2.0","id":1,"result":{...}}
```

**Error format**:

```json
{ "jsonrpc": "2.0", "id": 1, "error": { "code": -32600, "message": "..." } }
```

## Building

```bash
# Build all bridges (incremental)
./xtask.sh build-js

# Force rebuild all
./xtask.sh build-js --force

# Build specific bridge manually
cd projects/cli-js-bridges/config-bridge
node --experimental-strip-types build.mts
```

## Output Locations

| Package       | Output                                         |
| ------------- | ---------------------------------------------- |
| bridge-core   | `bridge-core/dist/*.js`                        |
| bridge-nodejs | `bridge-nodejs/dist/*.js`                      |
| bridge-bun    | `bridge-bun/dist/*.js`                         |
| bridge-deno   | `bridge-deno/dist/*.js`                        |
| config-bridge | `projects/cli/assets/js/dist/config-bridge.js` |

## Testing

```bash
# Manual test via stdin
export NSAGA_CLI_CONFIG_PATH="/path/to/config.js"
echo '{"jsonrpc":"2.0","id":1,"method":"config.get","params":{}}' | \
  node projects/cli/assets/js/dist/config-bridge.js
```

## Common Issues

### Import Errors

- Use `@nsaga/bridge-*` package names, not relative paths
- Add transitive deps to `devDependencies` for TypeScript resolution

### Runtime Detection

The bridge reads `NSAGA_RUNTIME` to select correct adapter:

- Set by Rust CLI before spawning
- Values: `node`, `bun`, `deno`

### Logs vs Output

- **stderr**: All logs (console.error, debug info)
- **stdout**: ONLY JSON-RPC responses
- Mixing will break protocol!
