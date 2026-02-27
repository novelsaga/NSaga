# XTask Build System Guide

## Overview

XTask (`cargo xtask`) is NovelSaga's build automation system - a Rust-based task runner for building JS bridges, native binaries, WASM, and shared libraries across multiple platforms.

## Structure

```
xtask/src/
├── main.rs              # CLI entry point with clap subcommands
├── tasks/
│   ├── mod.rs           # Task module exports
│   ├── build.rs         # JS bridge builds (config-bridge, etc.)
│   ├── bindings.rs      # TypeScript bindings generation (ts-rs)
│   ├── cli.rs           # CLI binary builds (cross-platform)
│   ├── clean.rs         # Clean build artifacts
│   ├── diplomat.rs      # Diplomat FFI bindings (C++/WASM)
│   ├── so.rs            # Shared library builds (.so/.dll/.dylib)
│   ├── test/            # Test-related tasks
│   │   ├── mod.rs
│   │   └── cli.rs       # E2E test runner
│   ├── utils.rs         # Shared utilities
│   └── wasm.rs          # WASM target builds
```

## Commands

### Build Commands

| Command | Purpose | Output |
|---------|---------|--------|
| `cargo xtask build-js` | Build JS bridges | `projects/cli/assets/js/dist/` |
| `cargo xtask build-js --force` | Force rebuild JS bridges | Same as above |
| `cargo xtask cli` | Build CLI for current platform | `out/cli/novelsaga` |
| `cargo xtask cli --target linux-arm64` | Cross-compile CLI | `out/cli/<target>/` |
| `cargo xtask cli-all` | Build CLI for all platforms | Multiple targets |
| `cargo xtask wasm` | Build WASM from core | `out/wasm/` |
| `cargo xtask so` | Build shared library (current) | `out/so/` |
| `cargo xtask so-all` | Build SO for all platforms | Multiple targets |
| `cargo xtask build-all` | Build everything | All outputs |

### Code Generation

| Command | Purpose | Output |
|---------|---------|--------|
| `cargo xtask gen-ts-bindings` | Generate TS types from Rust | `projects/cli-js-bridges/config-bridge/src/types/` |
| `cargo xtask gen-diplomat-bindings` | Generate C++/TS FFI | `target/diplomat/` |

### Testing & Cleanup

| Command | Purpose |
|---------|---------|
| `cargo xtask e2e` | Run end-to-end tests |
| `cargo xtask clean-all` | Clean all build artifacts |
| `cargo xtask clean-wasm` | Clean WASM artifacts |
| `cargo xtask clean-so` | Clean SO artifacts |
| `cargo xtask clean-cli` | Clean CLI artifacts |
| `cargo xtask clean-js` | Clean JS bridge artifacts |

## Build Architecture

### JS Bridge Build Process (`tasks/build.rs`)

Layered dependency build (parallel within layers):

```
Layer 1: bridge-core (foundation)
Layer 2: bridge-nodejs, bridge-bun, bridge-deno (parallel)
Layer 3: config-bridge (depends on all above)
```

Each layer builds sequentially; bridges within a layer build in parallel via `thread::scope`.

### CLI Cross-Compilation (`tasks/cli.rs`)

Supports 6 targets:
- `linux-x64` / `linux-arm64`
- `macos-x64` / `macos-arm64`
- `windows-x64` / `windows-arm64`

Uses `cross` for cross-compilation when target ≠ host.

### TypeScript Bindings (`tasks/bindings.rs`)

1. Sets `TS_RS_EXPORT_DIR` environment variable
2. Runs `cargo test -p novelsaga-core export_bindings`
3. Adds `/* eslint-disable */` headers to generated files
4. Output: `_*.ts` files in config-bridge types directory

### Diplomat FFI (`tasks/diplomat.rs`)

Generates C++ and TypeScript bindings from `diplomat_ffi.rs`:
- C++: `target/diplomat/cpp/`
- TypeScript/WASM: `target/diplomat/ts/`

## Output Directories

| Target | Default Path | Override Env Var |
|--------|--------------|------------------|
| WASM | `out/wasm/` | `WASM_OUT_DIR` |
| SO | `out/so/` | `SO_OUT_DIR` |
| CLI | `out/cli/` | `CLI_OUT_DIR` |
| Core | `projects/core/` | `CORE_PROJECT_DIR` |
| CLI project | `projects/cli/` | `CLI_PROJECT_DIR` |
| Cargo target | `target/` | `CARGO_TARGET_DIR` |

## Key Patterns

### Adding a New Task

1. Create `tasks/my_task.rs`:
   ```rust
   use anyhow::Result;
   
   pub fn run() -> Result<()> {
     println!("Running my task...");
     Ok(())
   }
   ```

2. Export in `tasks/mod.rs`:
   ```rust
   pub mod my_task;
   ```

3. Add subcommand in `main.rs`:
   ```rust
   #[derive(Subcommand)]
   enum Commands {
     MyTask,
   }
   ```

4. Wire up in `main()`:
   ```rust
   Commands::MyTask => tasks::my_task::run()?,
   ```

### Using Utilities

```rust
use crate::tasks::utils::*;

let root = project_root();           // Workspace root
let wasm_dir = wasm_out_dir();       // WASM output
let cli_dir = cli_out_dir();         // CLI output
let platform = detect_platform()?;   // Current platform string
```

### Running External Commands

```rust
use crate::tasks::utils::run_command;
use std::process::Command;

run_command(
  Command::new("cargo")
    .args(["build", "--release"])
    .current_dir(&project_root()),
)?;
```

## Testing

Tests are inline in task files using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn test_something() {
    assert!(true);
  }
}
```

Run tests: `cargo test -p xtask`

## Anti-Patterns

| Forbidden | Do Instead |
|-----------|------------|
| Hardcode paths | Use `utils::project_root()` and path helpers |
| Ignore command failures | Always use `run_command()` or check `status.success()` |
| Build bridges directly without `build.rs` | Use `tasks::build::build_all()` |
| Skip `generate_types()` before JS build | Type generation is prerequisite |

## Notes

- **Asset embedding**: CLI embeds JS assets at compile time; release builds use `asset_install_dir`
- **Parallel builds**: JS bridges use `thread::scope` for parallelism; use sparingly elsewhere
- **Error handling**: All tasks return `anyhow::Result<()>` for consistent error propagation
- **CI integration**: Commands designed to work in headless CI environments (no TTY assumptions)
