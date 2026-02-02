# Core Module Guide

## Overview

The Core module (`novelsaga-core`) provides the foundational types, state management, and config system for NovelSaga.

## Structure

```
projects/core/src/
├── lib.rs                      # Crate root, exports
├── diplomat_ffi.rs             # Diplomat FFI bindings for C/WASM
├── article/
│   └── mod.rs                  # Article types (content + metadata)
├── config/
│   ├── mod.rs                  # Config types + ConfigManager
│   ├── file_def.rs             # CONFIG_FILE_NAMES, ignore patterns
│   ├── fileformat/
│   │   └── mod.rs              # NovelSagaFileFormat enum
│   ├── workspace.rs            # WorkspaceConfig (cache_dir, novelsaga_dir)
│   └── formatter.rs            # FormatterConfig (fmt settings)
├── library/
│   ├── mod.rs                  # Library utilities
│   └── formatter.rs            # Formatter implementation
└── state/
    ├── mod.rs                  # State module exports
    ├── _state.rs               # State struct definition
    ├── feat.rs                 # Feature flags
    ├── init.rs                 # Initializer - OnceLock singleton
    └── manager/
        ├── mod.rs              # Manager exports
        └── config.rs           # ConfigManager - config loading + caching
```

## Key Components

### Config Types

```rust
// Root config (workspace-level, immutable)
#[derive(Deserialize, TS)]
#[ts(export, export_to = "_config.ts")]
pub struct RootConfig {
    pub workspace: WorkspaceConfig,
}

// Overridable config (per-directory, layered)
#[derive(Deserialize, TS)]
#[ts(export, export_to = "_config.ts")]
pub struct OverridableConfig {
    pub fmt: FormatterConfig,
}

// Final merged config
pub struct NovelSagaConfig {
    #[serde(flatten)]
    pub root: RootConfig,
    #[serde(flatten)]
    pub overridable: OverridableConfig,
}
```

### ConfigManager (`state/manager/config.rs`)

Handles config discovery, loading, and caching:

```rust
// Initialize with loaders from CLI
let manager = ConfigManager::new()
    .with_js_loader(js_loader)
    .with_ts_loader(ts_loader);

// Get root config (searches upward from cwd)
let root = manager.get_root_config()?;

// Get override config for directory (cached)
let override_cfg = manager.get_override_config(&dir)?;

// Clear cache for hot-reload
manager.del_override_config_cache(&dir);
```

### Initializer (`state/init.rs`)

Global singleton pattern using `OnceLock`:

```rust
// MUST call init() before get()
Initializer::init(Features::default())?;

// Then access state anywhere
let state = Initializer::get();
```

### File Format Detection (`config/fileformat/mod.rs`)

```rust
pub enum NovelSagaFileFormat {
    Corn, Ron, Toml, Yaml, Json5, Json, Ini,  // Static
    Js, Mjs, Cjs,  // JavaScript
    Ts, Mts, Cts,  // TypeScript
}

// Get extensions for a format
let exts = NovelSagaFileFormat::Js.get_extensions();  // [".js"]

// Check if format needs JS runtime
let needs_js = format.is_js_family();  // true for Js, Mjs, Cjs
let needs_ts = format.is_ts_family();  // true for Ts, Mts, Cts
```

## Type Exports to TypeScript

Core types are exported to TypeScript via `ts-rs`:

1. Add derive macro:

   ```rust
   #[derive(TS)]
   #[ts(export, export_to = "_config.ts")]
   pub struct MyConfig { ... }
   ```

2. Run test to generate:

   ```bash
   cargo test export_bindings
   ```

3. Types appear in `projects/cli-js-bridges/config-bridge/src/types/_config.ts`

4. Extend (don't modify!) in separate file:
   ```typescript
   // config.ts
   import type { RootConfig, OverridableConfig } from './_config.ts'
   export type NovelSagaConfig = RootConfig & OverridableConfig
   ```

## Config File Discovery

Files searched (in order):

- `novelsaga.config.*`
- `.novelsaga.*`

Extensions (by priority):

1. Static: `.toml`, `.yaml`, `.json`, `.json5`, `.ron`, `.corn`, `.ini`
2. JavaScript: `.js`, `.mjs`, `.cjs` (requires JS loader)
3. TypeScript: `.ts`, `.mts`, `.cts` (requires TS loader + Node ≥23.6)

Search direction: Current directory → parent → ... → workspace root

## Testing

```bash
# All core tests
cargo test -p novelsaga-core

# Generate TS bindings
cargo test -p novelsaga-core export_bindings
```

## Common Tasks

### Adding a New Config Field

1. Add field to appropriate struct in `config/mod.rs`
2. Add `#[derive(TS)]` if exposing to TypeScript
3. Run `cargo test export_bindings`
4. Update JS bridge types if needed

### Adding a New File Format

1. Add variant to `NovelSagaFileFormat` enum
2. Update `get_extensions()` method
3. Add parsing logic in `ConfigManager`

### Modifying State

1. Edit state struct in `state/_state.rs`
2. Update `Initializer::init()` if new dependencies
3. Add accessor methods as needed
