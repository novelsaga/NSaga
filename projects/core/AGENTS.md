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
│   ├── workspace.rs            # WorkspaceConfig (cache_dir)
│   └── formatter.rs            # FormatterConfig (fmt settings)
├── library/
│   ├── mod.rs                  # Library utilities
│   └── formatter.rs            # Formatter implementation
├── metadata/
│   ├── mod.rs                  # Re-exports: MetadataEntity, MetadataQuery, QueryResult
│   ├── model.rs                # MetadataEntity struct + TS export
│   ├── parser.rs               # Type inference, namespace generation
│   └── query.rs                # MetadataQuery trait + InMemoryMetadataStore
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

### Metadata Module (`metadata/`)

Provides novel document metadata definition, parsing, and query interfaces.

#### MetadataEntity (`metadata/model.rs`)

Core data type for a metadata document — exported to TypeScript as `_metadata.ts`:

```rust
pub struct MetadataEntity {
    pub id: String,           // Unique identifier
    pub type_: String,        // e.g. "character", "scene", "note"
    pub namespace: String,    // e.g. "global", "book-01", "book-01/part-01"
    pub frontmatter: Value,   // serde_json::Value (Record<string, any> in TS)
    pub body: String,         // Markdown body content
}

// Construct
let entity = MetadataEntity::new(id, type_, namespace, frontmatter, body);

// Access frontmatter fields
entity.get_field("author");           // Option<&Value>
entity.get_type_from_frontmatter();   // Option<String>
```

#### Parser Utilities (`metadata/parser.rs`)

```rust
// Infer entity type from directory structure
infer_type_from_path(Path::new("metadata/characters/hero.md")) // "character"
infer_type_from_path(Path::new("metadata/scenes/open.md"))     // "scene"
infer_type_from_path(Path::new("metadata/notes/ch1.md"))       // "note"

// Resolve type: frontmatter takes priority over path inference
resolve_type(path, &frontmatter)  // uses frontmatter["type"] if present

// Generate namespace from file location relative to workspace root
generate_namespace(
    Path::new("/project/book-01/metadata/hero.md"),
    Path::new("/project"),
) // "book-01"
// Root-level metadata/ → "global"
// book-01/part-01/metadata/ → "book-01/part-01"
```

**Namespace rules**:

| Path | Namespace |
| ---- | --------- |
| `metadata/*.md` | `"global"` |
| `book-01/metadata/*.md` | `"book-01"` |
| `book-01/part-01/metadata/*.md` | `"book-01/part-01"` |

#### MetadataQuery Trait + InMemoryMetadataStore (`metadata/query.rs`)

```rust
// Trait defining the query interface
pub trait MetadataQuery {
    fn get_by_id(&self, id: &str) -> Option<MetadataEntity>;
    fn get_by_name(&self, name: &str, namespace: &str) -> Option<MetadataEntity>;
    fn list_by_type(&self, type_: &str, namespace: Option<&str>) -> Vec<MetadataEntity>;
    fn list_by_namespace(&self, namespace: &str) -> Vec<MetadataEntity>;
    fn search(&self, query: &str, type_filter: Option<&str>) -> Vec<MetadataEntity>;
}

// In-memory implementation (HashMap-backed, dual-indexed)
let mut store = InMemoryMetadataStore::new();
store.insert(entity);
store.get_by_id("hero-1");
store.list_by_type("character", Some("book-01"));
store.search("protagonist", None);
```
## Type Exports to TypeScript

Core types are exported to TypeScript via `ts-rs`.

### Adding a New Type

1. Add derive macro (**no `export` keyword**, only `export_to` with filename):

   ```rust
   #[derive(TS)]
   #[ts(export_to = "_mytype.ts")]
   pub struct MyType { ... }
   ```

2. Register `.export()` in `config/mod.rs` test:

   ```rust
   // In config::tests::export_bindings
   MyType::export().expect("failed to export MyType");
   ```

3. Generate via xtask only:

   ```bash
   cargo run -p xtask -- gen-ts-bindings
   ```

4. Types appear as separate `_*.ts` files in `projects/cli-js-bridges/config-bridge/src/types/`:
   - `_format_config.ts` - FormatConfig
   - `_workspace_config.ts` - WorkspaceConfig
   - `_root_config.ts` - RootConfig
   - `_overridable_config.ts` - OverridableConfig
   - `_novelsaga_config.ts` - NovelSagaConfig (internal, prefixed `_NovelSagaConfig`)
   - `_metadata.ts` - MetadataEntity

5. Consume via `config.ts` (do not modify `_*.ts` directly):
   ```typescript
   // config.ts
   import type { OverridableConfig } from './_overridable_config'
   import type { RootConfig } from './_root_config'
   export type NovelSagaConfig = OverridableConfig & RootConfig
   ```

6. Generated `_*.ts` files are gitignored — regenerate after Rust type changes.

### IMPORTANT: `export` vs `export_to`

- `#[ts(export, export_to = "...")]` — **FORBIDDEN**: auto-generates a test that writes files on every `cargo test`
- `#[ts(export_to = "_name.ts")]` — **CORRECT**: only exports when `.export()` is called explicitly
- All ts-rs file writes are controlled exclusively by `xtask gen-ts-bindings`

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
