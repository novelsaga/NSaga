---
name: cli-dev
description: CLI and bridge management development for NovelSaga - use when modifying CLI commands, bridge system, or runtime management in projects/cli
---

# NovelSaga CLI Development

## Quick Reference

- **Entry point**: `main.rs` - initializes BridgeManager, ConfigLoader, and dispatches commands
- **CLI args**: `args/mod.rs` - clap derive macros with global `--runtime`, `--node-path`, etc.
- **Bridge system**: Thread-safe, lazy-loading via `BridgeManager` in `bridge/manager/mod.rs`
- **Runtime discovery**: `bridge/runtime/discovery.rs` - finds Node.js/Bun/Deno with version detection
- **Tests in same file**: `#[cfg(test)] mod tests { ... }`

## CLI Structure

```
projects/cli/src/
├── main.rs              # Entry: BridgeManager + ConfigLoader init, command dispatch
├── args/mod.rs          # Cli struct with clap derive, Commands enum
├── bridge/
│   ├── manager/         # BridgeManager - lazy loading, thread-safe
│   ├── runtime/         # RuntimeDiscovery - Node/Bun/Deno detection
│   ├── rpc/             # RpcClient - JSON-RPC 2.0 communication
│   └── transport/       # StdioTransport - stdin/stdout communication
├── config/loader.rs     # ConfigLoader - creates JS/TS loaders for Core
└── assets/locator.rs    # AssetLocator - finds JS assets in dev/release
```

### Command Dispatch Pattern

```rust
// main.rs - command routing
match &cli.command {
  Some(Commands::Lsp {}) => { lsp::start().await; }
  Some(Commands::Init { path }) => { /* ... */ }
  Some(Commands::Format { files, check }) => { /* ... */ }
  Some(Commands::Check { files }) => { /* ... */ }
  None => { print_status_info(&cli); }
}
```

### CLI Arguments with clap

```rust
// args/mod.rs - derive-based CLI definition
#[derive(Parser, Clone)]
pub struct Cli {
  #[arg(long, default_value = "auto", global = true)]
  runtime: RuntimeChoice,

  #[command(subcommand)]
  pub command: Option<Commands>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
  Lsp {},
  Init { path: PathBuf },
  // ...
}
```

## Bridge Management

### BridgeManager - Thread-Safe, Lazy Loading

```rust
// Initialize once in main.rs
let bridge_manager = Arc::new(BridgeManager::new());

// Register a bridge factory
manager.register("config-bridge", || {
  let runtime = RuntimeDiscovery::find_best_runtime()?;
  let bridge = ConfigBridge::new(runtime, script_path, env)?;
  Ok(Box::new(bridge))
});

// Call methods (auto-spawns on first call)
let result = manager.call("config-bridge", "config.get", json!({}))?;
```

### Runtime Discovery

```rust
// Auto-detect best runtime (Node.js > Bun > Deno)
let runtime = RuntimeDiscovery::find_best_runtime()?;

// Or with user preference
let runtime = RuntimeDiscovery::find_runtime_with_preference(
    Some(RuntimeType::NodeJs),
    Some(PathBuf::from("/custom/node")),
)?;

// Check TypeScript support
if runtime.supports_native_typescript() {
  // Node.js ≥23.6 or Bun/Deno
}
```

### ConfigLoader - Bridge-to-Core Integration

```rust
// Creates loaders that use bridges for Core's ConfigManager
let config_loader = ConfigLoader::new(bridge_manager.clone(), &cli);

let js_loader = Some(config_loader.create_js_loader());
let ts_loader = Some(config_loader.create_ts_loader());

// Pass to Core
let feature = Feature::new(js_loader, ts_loader);
Initializer::init(feature)?;
```

## Environment Variables

Defined in `bridge/env_keys.rs` (synced with JS side):

| Variable                | Description                          |
| ----------------------- | ------------------------------------ |
| `NSAGA_RUNTIME`         | Current runtime type (node/bun/deno) |
| `NSAGA_CLI_CONFIG_PATH` | Config file path for config-bridge   |
| `NSAGA_WORKSPACE_ROOT`  | Workspace root directory             |

## Build Commands

| Task              | Command                                    |
| ----------------- | ------------------------------------------ |
| Build CLI         | `cargo build -p novelsaga-cli`             |
| Test CLI          | `cargo test -p novelsaga-cli`              |
| Integration tests | `cargo test -p novelsaga-cli -- --ignored` |
| E2E tests         | `xtask e2e`                                |

## Anti-Patterns

| Don't                                           | Do Instead                                                      |
| ----------------------------------------------- | --------------------------------------------------------------- |
| Create Bridge without `BridgeManager`           | Use `manager.register()` + `manager.call()` for lazy loading    |
| Call `RuntimeDiscovery` on every request        | Cache `RuntimeInfo` in bridge or use `BridgeManager`            |
| Hardcode runtime paths                          | Use `RuntimeDiscovery::find_best_runtime()` with fallback chain |
| Spawn bridge processes in tests without cleanup | Use `manager.shutdown_bridge()` or `manager.shutdown_all()`     |
| Mix LSP protocol handling with bridge logic     | Keep LSP in `lsp/` module, bridges in `bridge/` module          |
| Use `println!` for logs in bridges              | Use `tracing` or `eprintln!` for debug output                   |

## When to Use

Load this skill when working on:

- `projects/cli/src/**/*.rs` - CLI-specific code
- CLI command handling (`args/`, command dispatch in `main.rs`)
- Bridge system (`bridge/manager/`, `bridge/runtime/`)
- Runtime discovery and process management
- ConfigLoader integration between CLI and Core
- Asset location for JS bridges

**Differentiation from other skills:**

- Use `core-dev` for: Config management, state initialization, core library logic
- Use `cli-dev` for: CLI commands, bridge management, runtime discovery (this skill)
- Use `lsp-dev` for: LSP protocol handlers, language server backend
