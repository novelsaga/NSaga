---
name: core-dev
description: Rust core library development - state management, config, type exports
---

# NovelSaga Core Development

## Quick Reference

- **Edition 2024** with nightly features (`#![feature(mpmc_channel)]`)
- **Tests in same file**: `#[cfg(test)] mod tests { ... }` in `projects/core/src/**/*.rs`
- **Build core**: `cargo build -p novelsaga-core`
- **Test core**: `cargo test -p novelsaga-core`

## State Management

### Initializer Pattern (`projects/core/src/state/init.rs`)

Global singleton using `OnceLock`:

```rust
// ✅ CORRECT: Initialize before get
let state = Initializer::init(Feature::default())?;

// ❌ WRONG: Will return Err(InitError::Uninitialized)
let state = Initializer::get()?;
```

**Available methods:**

- `init(feature)` - Initialize with feature flags
- `get()` - Get state reference (fails if not initialized)
- `with_read(|state| { ... })` - Read access with closure
- `get_or_init_with(f)` - Lazy initialization

### ConfigManager (`projects/core/src/state/manager/config.rs`)

Handles config discovery, loading, and caching:

```rust
// Create manager with feature flags
let manager = ConfigManager::new(feature);

// Get root config (searches upward from cwd)
let root = manager.get_root_config();

// Get override config for directory (cached)
let override_cfg = manager.get_override_config(&dir)?;

// Clear cache for hot-reload
manager.del_override_config_cache(&dir);
```

## Type Exports

### ts-rs Pattern (`projects/core/src/config/mod.rs`)

Export Rust types to TypeScript:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "_config.ts")]
#[serde(default)]
pub struct MyConfig {
  pub field: String,
}
```

**Generated types location:** `projects/cli-js-bridges/config-bridge/src/types/_config.ts`

**DO NOT modify `_config.ts`** - extend in separate files:

```typescript
// config.ts
import type { RootConfig } from './_config.ts'
export type NovelSagaConfig = RootConfig & {
  /* extensions */
}
```

### Config Types Hierarchy

- `RootConfig` - Workspace-level, immutable
- `OverridableConfig` - Per-directory, layered overrides
- `NovelSagaConfig` - Final merged config

## Config System

### File Discovery

Files searched (priority order):

1. `novelsaga.config.*`
2. `.novelsaga.*`

Extensions (by priority):

1. Static: `.toml`, `.yaml`, `.json`, `.json5`, `.ron`, `.corn`, `.ini`
2. JavaScript: `.js`, `.mjs`, `.cjs` (requires JS loader)
3. TypeScript: `.ts`, `.mts`, `.cts` (requires TS loader + Node ≥23.6)

Search direction: Current directory → parent → ... → workspace root

### File Format Detection

```rust
// Check format capabilities
let needs_js = format.is_js_family();  // true for Js, Mjs, Cjs
let needs_ts = format.is_ts_family();  // true for Ts, Mts, Cts
let exts = NovelSagaFileFormat::Js.get_extensions();  // [".js"]
```

## Anti-Patterns

| Don't                                      | Do Instead                        |
| ------------------------------------------ | --------------------------------- |
| Separate test files                        | `#[cfg(test)]` in same file       |
| `Initializer::get()` without init          | Call `init()` first               |
| `println!` in libraries                    | Use `tracing` macros              |
| Modify `_config.ts` directly               | Extend in separate files          |
| Call `ConfigManager` without feature flags | Pass `Feature` from CLI           |
| Ignore `InitError` results                 | Handle with `?` or explicit match |

## When to Use

Load this skill when working on:

- `projects/core/src/**/*.rs` - Core library code
- State management and initialization patterns
- Configuration systems and file discovery
- Type exports to TypeScript via ts-rs
- `ConfigManager` and config loading logic
- Adding new config fields or file formats

**NOT for:** CLI/LSP server code (use `cli-dev` skill instead)
