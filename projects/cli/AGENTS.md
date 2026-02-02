# CLI Module Guide

## Overview

The CLI module (`novelsaga-cli`) provides the LSP server and JS Bridge system for NovelSaga.

## Structure

```
projects/cli/src/
├── main.rs                 # Entry point, CLI argument handling
├── args/mod.rs             # CLI argument definitions (clap)
├── config/
│   ├── mod.rs              # Config module exports
│   └── loader.rs           # ConfigLoader - creates js/ts loaders for Core
├── assets/
│   ├── mod.rs              # Asset management exports
│   ├── locator.rs          # AssetLocator - finds JS assets in dev/release
│   └── error.rs            # Asset-related errors
├── bridge/
│   ├── mod.rs              # Bridge system exports
│   ├── env_keys.rs         # Environment variable constants (synced with JS)
│   ├── error.rs            # Bridge error types
│   ├── manager/
│   │   ├── mod.rs          # BridgeManager - lazy loading, thread-safe
│   │   ├── bridge.rs       # Bridge trait + BridgeInfo
│   │   └── config_bridge.rs # ConfigBridge implementation
│   ├── rpc/
│   │   ├── mod.rs          # RPC module exports
│   │   ├── client.rs       # RpcClient - JSON-RPC 2.0 communication
│   │   ├── types.rs        # RPC request/response types
│   │   └── tests.rs        # RPC unit tests
│   ├── runtime/
│   │   ├── mod.rs          # Runtime module exports
│   │   ├── discovery.rs    # RuntimeDiscovery - finds Node/Bun/Deno
│   │   ├── process.rs      # RuntimeProcess - spawns and manages processes
│   │   └── tests.rs        # Runtime unit tests
│   └── transport/
│       ├── mod.rs          # Transport module exports
│       ├── stdio.rs        # StdioTransport - stdin/stdout communication
│       └── tests.rs        # Transport unit tests
├── lsp/
│   ├── mod.rs              # LSP server exports
│   └── backend.rs          # LSP backend implementation
└── plugins/
    ├── mod.rs              # Plugin system exports
    └── consts.rs           # Plugin-related constants
```

## Key Components

### ConfigLoader (`config/loader.rs`)

Creates `JsLoader` and `TsLoader` closures for Core's `ConfigManager`:

```rust
// Usage in main.rs
let loader = ConfigLoader::new(cli_args)?;
let js_loader = loader.create_js_loader();
let ts_loader = loader.create_ts_loader();

// Pass to Core
config_manager.set_js_loader(js_loader);
config_manager.set_ts_loader(ts_loader);
```

### BridgeManager (`bridge/manager/mod.rs`)

Thread-safe, lazy-loading bridge manager:

```rust
// Get or spawn a bridge
let bridge = manager.get_bridge("config-bridge")?;

// Call RPC method
let result = bridge.call("config.get", json!({"path": path}))?;
```

### RuntimeDiscovery (`bridge/runtime/discovery.rs`)

Finds JavaScript runtimes with version detection:

```rust
// Auto-detect best runtime
let runtime = RuntimeDiscovery::find_runtime()?;

// Or with user preference
let runtime = RuntimeDiscovery::find_runtime_with_preference(
    RuntimeType::Node,
    Some("/usr/local/bin/node"),
)?;
```

### AssetLocator (`assets/locator.rs`)

Finds JS bridge assets in development and release builds:

- **Development**: `projects/cli/assets/js/dist/`
- **Release**: Embedded in binary, extracted to `~/.cache/novelsaga/assets/<version>/`

## CLI Arguments

```bash
novelsaga [OPTIONS]

Options:
  --runtime <TYPE>     Runtime: auto, node, bun, deno (default: auto)
  --node-path <PATH>   Custom Node.js executable path
  --bun-path <PATH>    Custom Bun executable path
  --deno-path <PATH>   Custom Deno executable path
  --help               Show help
  --version            Show version
```

## Environment Variables

Defined in `bridge/env_keys.rs` (synced with JS side):

| Variable                | Description                          |
| ----------------------- | ------------------------------------ |
| `NSAGA_RUNTIME`         | Current runtime type (node/bun/deno) |
| `NSAGA_CLI_CONFIG_PATH` | Config file path for config-bridge   |
| `NSAGA_WORKSPACE_ROOT`  | Workspace root directory             |

## Testing

```bash
# Unit tests (in same files as source)
cargo test -p novelsaga-cli

# Integration tests
cargo test -p novelsaga-cli test_bridge_manager -- --ignored

# E2E tests (via xtask)
./xtask.sh e2e
```

## Common Tasks

### Adding a New Bridge

1. Create `bridge/manager/my_bridge.rs`
2. Implement the `Bridge` trait
3. Register in `BridgeManager::new()`
4. Add corresponding JS bridge in `projects/cli-js-bridges/`

### Modifying CLI Arguments

Edit `args/mod.rs` using clap derive macros:

```rust
#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "auto")]
    runtime: RuntimeType,
}
```

### Debugging Bridge Communication

Set log level and run manually:

```bash
RUST_LOG=debug cargo run -p novelsaga-cli -- --runtime node
```
