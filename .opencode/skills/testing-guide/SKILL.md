---
name: testing-guide
description: NovelSaga-specific testing patterns - same-file tests, E2E structure, test matrix for multiple runtimes and config formats
---

# NovelSaga Testing Guide

## Quick Reference

| Command                               | Purpose                             | Notes                                   |
| ------------------------------------- | ----------------------------------- | --------------------------------------- |
| `cargo test`                          | All Rust tests (unit + integration) | Ignores #[ignore] tests by default      |
| `cargo test -p novelsaga-core`        | Core library tests only             | Type export tests included              |
| `cargo test -p novelsaga-cli`         | CLI tests only                      | Includes bridge tests                   |
| `cargo test <name> -- --ignored`      | Run ignored integration tests       | Requires actual runtime (Node/Bun/Deno) |
| `cargo test -- --nocapture --ignored` | Run with output                     | Useful for debugging integration tests  |
| `xtask e2e`                           | End-to-end CLI tests                | Matrix: runtimes × config formats       |
| `xtask e2e --runtime node`            | E2E with specific runtime           | Filters to single runtime               |

## Same-File Tests (#[cfg(test)])

NovelSaga uses inline `#[cfg(test)]` modules at the bottom of source files.

### Pattern

```rust
// In projects/core/src/config/mod.rs or any .rs file

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_feature() {
        // Arrange
        let config = NovelSagaConfig::default();

        // Act
        let result = config.merge(&OverridableConfig::default());

        // Assert
        assert!(result.root.workspace.is_some());
    }
}
```

### Key Rules

1. **Module placement**: Always at file end after main code
2. **Imports**: Use `use super::*;` to access parent scope
3. **Naming**: `mod tests { }` (singular or plural acceptable)
4. **No separate files**: Test files MUST NOT exist in project structure

### Example from Core

```rust
// projects/core/src/config/mod.rs
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn export_bindings() {
    formatter::FormatConfig::export().expect("failed to export FormatConfig");
    workspace::WorkspaceConfig::export().expect("failed to export WorkspaceConfig");
    RootConfig::export().expect("failed to export RootConfig");
    OverridableConfig::export().expect("failed to export OverridableConfig");
    NovelSagaConfig::export().expect("failed to export NovelSagaConfig");
  }
}
```

## Integration Tests (Ignored Tests)

Tests that require external resources (runtime, filesystem, processes) use `#[ignore]`.

### Pattern

```rust
#[cfg(test)]
mod integration_tests {
    #[test]
    #[ignore = "requires actual runtime environment"]
    fn test_find_nodejs() {
        let discovery = RuntimeDiscovery::new();
        let result = discovery.find_runtime(RuntimeType::NodeJs)
            .expect("Discovery failed");

        assert!(result.is_some());
    }
}
```

### Why #[ignore]

- **Runtime dependency**: Requires Node.js, Bun, or Deno installed
- **File I/O**: Creates/reads temp directories during test
- **External processes**: Spawns child processes
- **CI/CD fragility**: Unpredictable in sandboxed environments

### Running Ignored Tests

```bash
# Run specific ignored test with output
cargo test test_find_nodejs -- --ignored --nocapture

# Run all ignored tests in a package
cargo test -p novelsaga-cli -- --ignored --nocapture

# Run tests matching pattern
cargo test config -- --ignored
```

## E2E Tests (xtask-based)

End-to-end tests run CLI with real bridges and config formats. Located in `xtask/src/tasks/test/cli.rs`.

### Test Matrix Structure

E2E tests systematically vary:

- **Runtimes**: Node.js, Bun, Deno (auto-detected, skipped if unavailable)
- **Config formats**: JS ESM (.mjs), JS CJS (.cjs), TS ESM (.mts), TS CJS (.cts)
- **Scenarios**: Config priority, error handling, format-specific features

### Matrix Example

```
Node.js × .mjs → ✓
Node.js × .cjs → ✓
Node.js × .mts → ✓ (requires Node ≥23.6)
Node.js × .cts → ✓ (requires Node ≥23.6)
Bun × .mjs → ✓
Bun × .cjs → ✓
Deno × .mjs → ✓
... (all combinations tested)
```

### Running E2E Tests

```bash
# Run all E2E tests (matrix)
xtask e2e

# Run E2E with specific runtime
xtask e2e --runtime node

# Filter by config format (internal only)
# (Modify cli.rs ConfigType enum filtering)

# With output for debugging
RUST_LOG=debug xtask e2e
```

### E2E Test Structure

```rust
// xtask/src/tasks/test/cli.rs
enum Runtime { Node, Bun, Deno }  // Auto-detected
enum ConfigType { JsEsm, JsCjs, TsEsm, TsCjs }  // Formats tested

pub fn run_e2e_tests() -> Result<()> {
    // 1. Ensure CLI built
    ensure_cli_built()?;

    // 2. Build bridges
    crate::tasks::build::build_all()?;

    // 3. Detect available runtimes
    let available_runtimes = Runtime::all_available();

    // 4. Run scenario × runtime × format combinations
    for runtime in available_runtimes {
        for config_type in [JsEsm, JsCjs, TsEsm, TsCjs] {
            // Create temp dir with config file
            // Run CLI with config
            // Verify results
        }
    }
}
```

### Writing E2E Scenarios

Each scenario tests a specific feature across the matrix:

```rust
fn test_config_priority(
    runtime: Runtime,
    config_type: ConfigType,
    temp_dir: &Path,
) -> Result<()> {
    // Create config in temp_dir with specified type
    fs::write(
        temp_dir.join(format!("novelsaga.config.{}", config_type.extension())),
        config_type.content(),
    )?;

    // Run CLI
    let output = Command::new("novelsaga")
        .arg("check")
        .env("NSAGA_RUNTIME", runtime.executable())
        .current_dir(temp_dir)
        .output()?;

    // Assert
    assert!(output.status.success(), "config check failed");
    Ok(())
}
```

## Type Export Tests

Core exports Rust types to TypeScript via `ts-rs`. Export tests must run before TypeScript builds.

### Pattern

```rust
// projects/core/src/config/mod.rs
#[cfg(test)]
mod tests {
    #[test]
    fn export_bindings() {
        // Export all types
        FormatterConfig::export().expect("failed to export");
        RootConfig::export().expect("failed to export");
        NovelSagaConfig::export().expect("failed to export");
    }
}
```

### Generated Output

```
projects/cli-js-bridges/config-bridge/src/types/_config.ts
```

### Running Export Tests

```bash
# Generate TS bindings
cargo test -p novelsaga-core export_bindings

# Full test run includes export
cargo test -p novelsaga-core
```

## Anti-Patterns

### ❌ Separate Test Files

**WRONG** - Creating `tests/lib.rs` or `tests/config_tests.rs`:

```
projects/core/
├── src/
│   └── config/mod.rs
└── tests/  ← DO NOT CREATE
    └── config_test.rs
```

**WHY**: Decouples tests from implementation, harder to maintain, violates convention.

**FIX**: Keep tests in same file using `#[cfg(test)] mod tests { }`

### ❌ Using #[ignore] Without Documentation

**WRONG**:

```rust
#[test]
#[ignore]  // ← Why? What does it need?
fn test_something() { }
```

**FIX**:

```rust
#[test]
#[ignore = "requires actual Node.js runtime environment"]
fn test_something() { }
```

The string explains WHAT resource is required, making it clear when to run with `-- --ignored`.

### ❌ Blocking On Runtime Features in Regular Tests

**WRONG** - Spawning processes in unit tests:

```rust
#[test]  // ← Will fail if runtime missing
fn test_runtime_detection() {
    let runtime = RuntimeDiscovery::find_runtime()?;  // May not exist
    assert!(runtime.is_some());
}
```

**FIX** - Mark as ignored:

```rust
#[test]
#[ignore = "requires Node/Bun/Deno installation"]
fn test_runtime_detection() {
    let runtime = RuntimeDiscovery::find_runtime()?;
    assert!(runtime.is_some());
}
```

Regular tests run in CI/CD with predictable environment.

### ❌ Tempfile Without Cleanup

**WRONG**:

```rust
let dir = tempfile::tempdir()?;
let path = dir.path();
// ← dir dropped before test ends, cleanup unpredictable
```

**FIX**:

```rust
let dir = tempfile::tempdir()?;
let path = dir.path();

// Use dir throughout test
fs::write(path.join("config.toml"), "...")?;

// dir automatically cleaned up at end of test scope
```

The `tempfile::tempdir()` returns `TempDir` with RAII cleanup.

## When to Use

### Same-File Tests

- Unit tests for functions/types
- Logic tests (no I/O, no external processes)
- Fast feedback (run with `cargo test`)
- Type export validation

### Ignored Integration Tests

- Runtime detection tests
- Real process spawning
- FileSystem operations (when tempfile isn't enough)
- Bridge communication with actual runtimes

**Command**: `cargo test <name> -- --ignored --nocapture`

### E2E Tests

- Full CLI workflow with bridges
- Multiple runtime compatibility
- Config file parsing across formats (.mjs, .mts, .cjs, .cts)
- Error scenarios with real environment

**Command**: `xtask e2e`

### Quick Dev Cycle

```bash
# Change source
vim projects/core/src/config/mod.rs

# Test immediately (unit tests only, fast)
cargo test -p novelsaga-core

# Test CLI with specific test
cargo test -p novelsaga-cli test_manager -- --ignored --nocapture

# Full E2E (slower, but comprehensive)
xtask e2e
```

## Environment Variables for Tests

| Variable         | Purpose                   | Example                                    |
| ---------------- | ------------------------- | ------------------------------------------ |
| `RUST_LOG`       | Logging level in tests    | `RUST_LOG=debug cargo test -- --nocapture` |
| `RUST_BACKTRACE` | Panic backtrace           | `RUST_BACKTRACE=1 cargo test`              |
| `NSAGA_RUNTIME`  | Override detected runtime | `NSAGA_RUNTIME=node xtask e2e`             |

## Type Export Workflow

1. **Define type with derive**:

   ```rust
   #[derive(TS)]
   #[ts(export, export_to = "_config.ts")]
   pub struct MyConfig { ... }
   ```

2. **Run export test**:

   ```bash
   cargo test -p novelsaga-core export_bindings
   ```

3. **Verify output exists**:

   ```bash
   ls projects/cli-js-bridges/config-bridge/src/types/_config.ts
   ```

4. **Never modify `_config.ts`** - it's auto-generated. Extend with separate file:
   ```typescript
   // projects/cli-js-bridges/config-bridge/src/types/config.ts
   import type { MyConfig } from './_config.ts'
   export type ExtendedConfig = MyConfig & { custom: string }
   ```
