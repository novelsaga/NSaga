---
name: ts-bridge
description: TypeScript bridge development for NovelSaga CLI JSON-RPC services - use when modifying projects/cli-js-bridges
---

# TypeScript Bridge Development

## Quick Reference

- **ESM modules**: `"type": "module"` in package.json
- **JSON-RPC 2.0**: Format `"service.method"` for routing
- **Logs to stderr**: Never `console.log()` in bridges (stdout is for JSON-RPC)

## Architecture

```
CLI (Rust) ←--JSON-RPC--→ Bridge (TS) ←--→ Runtime (Node/Bun/Deno)
```

## Service Pattern

```typescript
import { createBridgeServer } from '@nsaga/bridge-core'

const server = createBridgeServer()

server.registerService('config', {
  async load(params: LoadParams): Promise<Config> {
    // Implementation
  },
})

await server.start()
```

## Build Commands

| Task              | Command                       |
| ----------------- | ----------------------------- |
| Incremental build | `./xtask.sh build-js`         |
| Force rebuild     | `./xtask.sh build-js --force` |

## Output Locations

- Package dists: `projects/cli-js-bridges/*/dist/`
- CLI assets: `projects/cli/assets/js/dist/`

## Anti-Patterns

| Don't                            | Do Instead                 |
| -------------------------------- | -------------------------- |
| `npm install` in subproject      | `pnpm install` at root     |
| Relative imports outside rootDir | Use `@nsaga/bridge-*`      |
| `console.log()` in bridges       | `console.error()` for logs |
| Modify `_config.ts`              | Extend in separate files   |

## When to Use

Load this skill when working on:

- `projects/cli-js-bridges/**/*.ts`
- JSON-RPC service implementations
- Bridge runtime adapters
