---
name: end-to-end-workflow
description: Decision tree for feature development - use to determine which files to modify for different feature types in NovelSaga
---

# End-to-End Feature Development Workflow

## Quick Reference

| Feature Type           | Files to Touch                                       | Build Order                                                     |
| ---------------------- | ---------------------------------------------------- | --------------------------------------------------------------- |
| **New Config Field**   | Core types → TS export → Bridge service              | `cargo test export_bindings` → `xtask build-js` → `cargo build` |
| **New CLI Command**    | CLI args → main dispatch → (optional) bridge         | `cargo build -p novelsaga-cli`                                  |
| **New Bridge Service** | Bridge service → Core integration → CLI registration | `xtask build-js` → `cargo build`                                |
| **New LSP Capability** | LSP backend → Core integration                       | `cargo build -p novelsaga-cli`                                  |
| **New File Format**    | Core format enum → ConfigManager parser              | `cargo build -p novelsaga-core`                                 |

## Decision Tree

### Starting Point: What Are You Building?

```
┌─────────────────────────────────────────────────────────────────────┐
│  FEATURE TYPE DECISION TREE                                         │
└─────────────────────────────────────────────────────────────────────┘

Q: Does it involve configuration types?
├── YES → Go to [Config Workflow](#config-workflow)
│
└── NO → Q: Does it add a CLI command?
    ├── YES → Go to [CLI Workflow](#cli-workflow)
    │
    └── NO → Q: Does it need JS bridge integration?
        ├── YES → Go to [Bridge Workflow](#bridge-workflow)
        │
        └── NO → Q: Does it modify LSP behavior?
            ├── YES → Go to [LSP Workflow](#lsp-workflow)
            │
            └── NO → Q: Is it a new library feature?
                ├── YES → Go to [Core Library Workflow](#core-library-workflow)
                │
                └── NO → See [other skills](#cross-references)
```

### Config Workflow

**Use when**: Adding/modifying config fields, new config file formats

**Files to modify**:

```
1. projects/core/src/config/mod.rs
   └─ Add field to RootConfig or OverridableConfig
   └─ Add #[derive(TS)] #[ts(export, export_to = "_config.ts")]

2. projects/core/src/config/formatter.rs (if fmt-related)
   └─ Add field to FormatterConfig

3. Run: cargo test export_bindings
   └─ Generates: projects/cli-js-bridges/config-bridge/src/types/_config.ts

4. (Optional) Extend types in projects/cli-js-bridges/config-bridge/src/types/config.ts
   └─ DO NOT modify _config.ts (generated)

5. (Optional) Update bridge service to use new fields
   └─ projects/cli-js-bridges/config-bridge/src/services/config.ts
```

### CLI Workflow

**Use when**: Adding/modifying CLI commands or arguments

**Files to modify**:

```
1. projects/cli/src/args/mod.rs
   └─ Add variant to Commands enum
   └─ Add fields with #[arg(...)] attributes

2. projects/cli/src/main.rs
   └─ Add match arm in command dispatch
   └─ Implement command logic (or delegate to module)

3. (Optional) Add module for complex commands
   └─ projects/cli/src/commands/<command>.rs
   └─ Export in projects/cli/src/commands/mod.rs
```

### Bridge Workflow

**Use when**: Adding a new JS bridge service or modifying bridge communication

**Files to modify**:

```
For NEW bridge service:

1. Create: projects/cli-js-bridges/<name>-bridge/
   ├─ package.json (name: "@nsaga/<name>-bridge")
   ├─ build.mts (uses @nsaga/build-tool)
   └─ src/index.ts (entry point)
   └─ src/services/<name>.ts (service implementation)

2. Rust side: projects/cli/src/bridge/manager/
   └─ Create <name>_bridge.rs implementing Bridge trait
   └─ Register in BridgeManager::new()

3. Build: xtask build-js

For EXISTING bridge modifications:

1. Service logic: projects/cli-js-bridges/<name>-bridge/src/services/*.ts
2. Type definitions: projects/cli-js-bridges/<name>-bridge/src/types/*.ts
3. Build: xtask build-js
```

### LSP Workflow

**Use when**: Adding LSP capabilities (formatting, hover, diagnostics, etc.)

**Files to modify**:

```
1. projects/cli/src/lsp/backend.rs
   └─ Implement LanguageServer trait methods
   └─ Register capabilities in initialize()

2. (Optional) Add document handlers:
   └─ did_open, did_change, did_close
   └─ Store in: self.documents Arc<RwLock<HashMap<Url, String>>>

3. (Optional) Formatting implementation:
   └─ Call Core library: novelsaga_core::library::formatter::format_text()
   └─ Return TextEdit with full document replacement
```

### Core Library Workflow

**Use when**: Adding new library functionality (formatters, parsers, utilities)

**Files to modify**:

```
1. projects/core/src/library/
   └─ Create: <feature>/mod.rs
   └─ Export in: projects/core/src/library/mod.rs

2. (Optional) Add FFI exposure:
   └─ projects/core/src/diplomat_ffi.rs
   └─ Wrap with #[diplomat::opaque]

3. Add tests in same file:
   └─ #[cfg(test)] mod tests { ... }
```

## Type Export Workflow

### Core → TypeScript Flow

```
┌─────────────────────────────────────────────────────────────┐
│  TYPE EXPORT PIPELINE                                       │
└─────────────────────────────────────────────────────────────┘

Step 1: Rust Core
projects/core/src/config/*.rs
├─ #[derive(TS)]
├─ #[ts(export, export_to = "_config.ts")]
└─ struct MyConfig { ... }

        ↓ cargo test export_bindings

Step 2: Generated Types
projects/cli-js-bridges/config-bridge/src/types/_config.ts
└─ Auto-generated: DO NOT MODIFY

        ↓ (manual) Extend in

Step 3: Bridge Types
projects/cli-js-bridges/config-bridge/src/types/config.ts
└─ export type NovelSagaConfig = RootConfig & OverridableConfig

        ↓ xtask build-js

Step 4: Bridge Distribution
projects/cli/assets/js/dist/config-bridge.js
└─ Bundled for CLI embedding

        ↓ cargo build

Step 5: CLI Binary
out/novelsaga (or target/debug/novelsaga)
└─ Embeds bridge assets
```

### Type Export Rules

| Rule           | Correct                      | Incorrect             |
| -------------- | ---------------------------- | --------------------- |
| Derive macro   | `#[derive(TS)]`              | Manual TS definitions |
| Export path    | `export_to = "_config.ts"`   | Custom paths          |
| Generated file | Read-only `_config.ts`       | Editing `_config.ts`  |
| Extensions     | Create `config.ts`           | Modify `_config.ts`   |
| Regeneration   | `cargo test export_bindings` | Hand-editing          |

## Build Order

### Correct Sequence for Type Changes

```bash
# 1. Modify Rust types with #[derive(TS)]
#    Edit: projects/core/src/config/*.rs

# 2. Generate TypeScript types
cargo test -p novelsaga-core export_bindings

# 3. Build JS bridges (includes type check)
xtask build-js

# 4. Build CLI with embedded bridges
cargo build -p novelsaga-cli

# 5. Run tests
cargo test -p novelsaga-cli
```

### Incremental Build Shortcuts

| What Changed    | Command                                        |
| --------------- | ---------------------------------------------- |
| Only Rust core  | `cargo build -p novelsaga-core`                |
| Only CLI        | `cargo build -p novelsaga-cli`                 |
| Only JS bridges | `xtask build-js`                               |
| Types + bridges | `cargo test export_bindings && xtask build-js` |
| Full rebuild    | `xtask build-all`                              |

### Build Dependencies Graph

```
novelsaga-core
    ↓ (dependency)
novelsaga-cli
    ↓ (spawns + embeds)
config-bridge (JS)
    ↓ (imports types from)
_config.ts (generated from core)
```

## Anti-Patterns

| Category            | Don't                               | Do Instead                                                            |
| ------------------- | ----------------------------------- | --------------------------------------------------------------------- |
| **Type Export**     | Edit `_config.ts` directly          | Extend in separate files, regenerate via `cargo test export_bindings` |
| **Build Order**     | Build CLI before bridges            | Run `xtask build-js` before `cargo build` after type changes          |
| **Bridge Creation** | Create bridge without BridgeManager | Register in manager for lazy-loading                                  |
| **CLI Args**        | Hardcode values in main.rs          | Use `args/mod.rs` with clap derive                                    |
| **JS Bridge**       | `console.log()` in production       | Use `console.error()` for logs, stdout for JSON-RPC                   |
| **Config Types**    | Add TS-only types in bridge         | Add to Rust core with `#[derive(TS)]`                                 |
| **Testing**         | Skip `cargo test export_bindings`   | Always regenerate after type changes                                  |
| **Dependencies**    | Install deps in subproject          | `pnpm install` at root only                                           |

## Cross-References

For implementation details, see specific skills:

| Topic                     | Skill           | Files                               |
| ------------------------- | --------------- | ----------------------------------- |
| Core types, config, state | `core-dev`      | `projects/core/src/**/*.rs`         |
| CLI commands, bridge mgmt | `cli-dev`       | `projects/cli/src/**/*.rs`          |
| LSP protocol, handlers    | `lsp-dev`       | `projects/cli/src/lsp/**/*.rs`      |
| JS bridge services        | `ts-bridge`     | `projects/cli-js-bridges/**/*.ts`   |
| Diplomat FFI              | `ffi-diplomat`  | `projects/core/src/diplomat_ffi.rs` |
| Nix builds                | `nix-workflow`  | `flake.nix`, `packages.nix`         |
| Testing patterns          | `testing-guide` | Test patterns across project        |

## When to Use

Load this skill when:

- Starting a new feature and unsure which files to modify
- Planning changes that span multiple modules (Core → TS → Bridge)
- Need to understand the build sequence after type changes
- Adding config fields that need to be exposed to TypeScript
- Creating a new bridge service

**This skill is for FILE MAPPING, not implementation.** For how to implement specific patterns, load the appropriate module skill.
