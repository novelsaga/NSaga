---
name: project-architecture
description: NovelSaga module relationships, data flow, and system initialization sequence - understand how Core, CLI, and JS bridges interact
---

# NovelSaga Project Architecture

## Quick Reference

**NovelSaga** is a three-tier system:

- **Core** (`projects/core/`) - Rust library providing state management, config system, article types
- **CLI** (`projects/cli/`) - Rust binary with LSP server and bridge manager
- **JS Bridges** (`projects/cli-js-bridges/`) - TypeScript JSON-RPC services for dynamic config loading

## Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│  EDITOR (VSCode, Neovim, etc.)                              │
│  ↓ LSP Protocol (JSON-RPC)                                  │
└──────────────────────┬──────────────────────────────────────┘
                       │
                ┌──────▼──────┐
                │   CLI Binary│
                │ (novelsaga) │
                └──────┬──────┘
                       │
        ┌──────────────┼──────────────┐
        │              │              │
   ┌────▼──┐    ┌────▼──┐    ┌──────▼────┐
   │  Core │    │Bridge │    │   LSP     │
   │ State │    │Manager│    │ Backend   │
   │  Init │    │       │    │           │
   └────┬──┘    └────┬──┘    └───────────┘
        │            │
        │      ┌─────▼──────────┐
        │      │ JS Bridges     │
        │      │(via stdin/out) │
        │      │                │
        │  ┌───┼───┬───────┬────┴─────┐
        │  │   │   │       │          │
        │  │   ▼   ▼       ▼          ▼
        │  │  Node Bun    Deno   bridge-core
        │  │ (runtime adapters)  (shared code)
        │  │
        │  └─────────► config-bridge
        │    (loads JS/TS configs)
        │
        └──────────────────────────────┘
              (state mgmt, config types)
```

### Relationships

| Module               | Depends On                    | Purpose                                            |
| -------------------- | ----------------------------- | -------------------------------------------------- |
| **CLI**              | Core, JS Bridges              | Entry point; orchestrates state init, bridges, LSP |
| **Core**             | None                          | Foundation; types, state, config loading interface |
| **JS Bridges**       | bridge-core, runtime adapters | Dynamic config loading via JSON-RPC                |
| **bridge-core**      | None                          | Shared RPC protocol, service interface             |
| **Runtime adapters** | bridge-core                   | Runtime-specific stdin/stdout I/O                  |
| **config-bridge**    | JS bridges, Core types        | Loads configs; reads NSAGA_RUNTIME env var         |

## Data Flow Diagram

### Initialization Sequence

```
1. Editor starts novelsaga LSP
   ↓
2. CLI main.rs:
   ├─ Parse CLI args
   ├─ Create BridgeManager (lazy-load JS bridges)
   ├─ Create ConfigLoader (bridges + runtime detection)
   ├─ Initialize Core with loaders
   │  ├─ Initializer::init(Feature::with_loaders(...))
   │  ├─ ConfigManager created
   │  └─ State stored in OnceLock
   └─ Start LSP server

3. LSP server (lsp/backend.rs):
   ├─ Receive LSP requests from editor
   ├─ Use Core state + config
   ├─ Send LSP responses back
   └─ (Optional) Call bridges for dynamic config
```

### Config Loading Flow

```
Editor requests document format
   ↓
LSP backend calls Initializer::get()
   ↓
ConfigManager::get_override_config(file_path)
   ├─ Check local cache first
   ├─ If not cached:
   │  ├─ Search upward from file_path
   │  ├─ Find .novelsaga.* or novelsaga.config.*
   │  ├─ Determine file format (ext check)
   │  │
   │  ├─ If static format (.toml, .json, etc):
   │  │  └─ Parse directly
   │  │
   │  ├─ If .js/.mjs/.cjs:
   │  │  └─ Call JS loader closure
   │  │     └─ BridgeManager.call("config-bridge", "config.get", ...)
   │  │        └─ config-bridge spawns JS process
   │  │           └─ Loads & returns config
   │  │
   │  └─ If .ts/.mts/.cts:
   │     └─ Call TS loader closure
   │        └─ Same RPC flow as JS
   │
   └─ Return config (cached for future calls)
```

### Bridge Communication Flow

```
Rust (CLI) Process                    JS Process (config-bridge)
    ↓                                        ↓
ConfigLoader.create_js_loader()  ←spawn─→  config-bridge.js
    ↓                                        ↓
BridgeManager.call()                      BridgeServer.handle()
    ↓                                        ↓
RpcClient.request()                       RpcHandler.dispatch()
    ↓                                        ↓
StdioTransport.write()  ─[JSON-RPC]→  StdioTransport.read()
    ↓                                        ↓
JSON: {"method":"config.get"...}  →  ConfigService.get()
    ↓                                        ↓
StdioTransport.read()   ←[JSON-RPC]─  StdioTransport.write()
    ↓                                        ↓
RpcClient.response()                      Return config
    ↓
Return parsed config to Core
```

## State Initialization Sequence

### Timeline: From Process Start to Ready

```
┌─────────────────────────────────────────────────────────────┐
│ 1. STARTUP (main.rs)                                        │
├─────────────────────────────────────────────────────────────┤
│  • Parse CLI args (--runtime, --node-path, etc.)            │
│  • Create BridgeManager (empty, no processes spawned)       │
│  • Create ConfigLoader                                      │
│  │                                                           │
│  └─► ConfigLoader::new(bridge_manager, cli_args)            │
│      ├─ Detects JS/TS loader support                        │
│      └─ Creates closures for loading JS/TS configs          │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. CORE INITIALIZATION (Initializer::init)                  │
├─────────────────────────────────────────────────────────────┤
│  • Create Feature with JS/TS loaders                        │
│  • Initializer::init(Feature) → OnceLock<State>             │
│  │                                                           │
│  └─► State contains:                                         │
│      ├─ ConfigManager (with loaders attached)               │
│      ├─ Feature flags                                       │
│      └─ Shared across entire CLI lifetime                   │
│                                                              │
│  ⚠️  CRITICAL: Must call init() BEFORE get()                │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. LSP SERVER START (lsp::start)                            │
├─────────────────────────────────────────────────────────────┤
│  • Initialize LSP connection                                │
│  • Register document handlers                               │
│  • Wait for editor requests                                 │
│                                                              │
│  On first request:                                           │
│  ├─ Call Initializer::get() (already init'd)                │
│  ├─ Use ConfigManager to load file config                   │
│  │  └─ May spawn JS bridge on first .js/.ts config         │
│  └─ Return formatted document or diagnostic                 │
└─────────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. RUNTIME (ongoing)                                        │
├─────────────────────────────────────────────────────────────┤
│  • BridgeManager keeps JS processes alive                   │
│  • ConfigManager caches loaded configs                      │
│  • LSP responds to editor requests                          │
│  • Call manager.shutdown_bridge() on change/reload          │
└─────────────────────────────────────────────────────────────┘
```

### Module Initialization Dependencies

```
Initializer::init(Feature)
    ↓
Creates ConfigManager
    ├─ Stores JS loader (from ConfigLoader)
    ├─ Stores TS loader (from ConfigLoader)
    └─ Initializes empty config cache

ConfigLoader
    ├─ Stores BridgeManager reference
    ├─ Detects available runtimes
    ├─ Creates JS loader closure
    │  └─ Closure captures BridgeManager
    │     └─ Will call config-bridge on first .js config
    └─ Creates TS loader closure
       └─ Closure captures BridgeManager
          └─ Will call config-bridge on first .ts config

BridgeManager (initialized in main.rs)
    ├─ Registers "config-bridge"
    ├─ Lazy-loads JS processes on first call
    ├─ Maintains RPC connection pool
    └─ Cleans up processes on shutdown
```

## Anti-Patterns

| Category                 | ❌ Don't Do                                                         | ✅ Do Instead                                                   |
| ------------------------ | ------------------------------------------------------------------- | --------------------------------------------------------------- |
| **State Access**         | Call `Initializer::get()` without `init()` first                    | Always: `Initializer::init(feature)` → then `get()`             |
| **Bridge Spawning**      | Create bridges directly in ConfigManager                            | Use `BridgeManager.register()` + `call()` for lazy-load         |
| **Config Caching**       | Clear cache on every config access                                  | Cache in ConfigManager; only clear on hot-reload                |
| **Type Generation**      | Modify `projects/cli-js-bridges/config-bridge/src/types/_config.ts` | Extend in separate `.ts` files; regenerate via `xtask build-js` |
| **Loader Closures**      | Capture mutable state in loaders                                    | Closures must be immutable `Fn` (not `FnMut`)                   |
| **CLI Initialization**   | Skip ConfigLoader; call ConfigManager directly                      | Use ConfigLoader → Feature → Initializer flow                   |
| **Bridge Communication** | Mix stdout/stderr for logs                                          | stderr for logs only; stdout for JSON-RPC responses only        |
| **Process Lifecycle**    | Spawn new JS process per config load                                | Use BridgeManager pooling; call `shutdown_bridge()` explicitly  |

## When to Use

Load this skill when:

- **Understanding system flow**: How Core, CLI, and bridges work together
- **Modifying initialization**: Changes to state setup, loader registration
- **Adding features affecting multiple modules**: New config fields, state types
- **Debugging cross-module issues**: Why is a loader not being called? Why is config stale?
- **Reviewing architecture decisions**: What layer should this logic live in?
- **Onboarding**: Getting familiar with module relationships

**When to use other skills instead:**

- **Core logic only** → Use `core-dev` skill
- **CLI/Bridge manager** → Use `cli-dev` skill
- **JS bridge services** → Use `ts-bridge` skill
- **LSP protocol** → Use `lsp-dev` skill
- **Build system** → Use `nix-build` or project-specific build skills
