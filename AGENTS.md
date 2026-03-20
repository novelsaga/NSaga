# NOVELSAGA PROJECT KNOWLEDGE BASE

## Golden Rules

1. **Never `npm install` in subprojects** — use `pnpm install` at workspace root only.
2. **Always use `node:` prefix for built-ins** — e.g., `node:fs` not `fs`.
3. **Call `Initializer::init()` before `Initializer::get()`** — state must be initialized.
4. **Print to stderr, never stdout in bridges** — stdout is reserved for JSON-RPC protocol.
5. **Keep tests in source files** — never create separate `.test.rs` files.
6. **Import order: std → external → crate** — follow `rustfmt.toml` grouping.
7. **Use `thiserror` for libraries, `anyhow` for apps** — structured errors vs. propagation.
8. **Use TDD for development** — write failing test first, then minimal code to pass, then refactor.
9. **Use git-worktree for large changes** — prevent accidental data loss on extensive refactors.
10. **Disclose AI assistance in commits** — add `Co-authored-by:` trailer when using AI tools.

---

## Project Skeleton

```
nsaga/
├── projects/
│   ├── core/              # Rust core: state, config, domain logic
│   │   └── src/
│   │       ├── metadata/  # Entity models, parsing, query traits
│   │       ├── state/     # Global state, initialization
│   │       └── config/    # Configuration types
│   ├── cli/               # Rust LSP server & CLI
│   │   └── src/
│   │       ├── bridge/    # JS bridge management, RPC
│   │       ├── lsp/       # LSP backend implementation
│   │       ├── metadata/  # IndexManager, CacheManager, etc.
│   │       └── commands/  # CLI command handlers
│   └── cli-js-bridges/    # TypeScript bridges (excluded from workspace)
│       ├── bridge-core/
│       ├── bridge-nodejs/
│       ├── bridge-bun/
│       └── bridge-deno/
├── xtask/                 # Build automation tasks
├── .cargo/
│   └── config.toml        # Cargo aliases (xtask)
├── flake.nix              # Nix development environment & hooks
└── out/                   # Build outputs
```

---

## Development Workflow

### Environment Setup

```bash
# Install JavaScript dependencies
pnpm install                    # Run at workspace root only

# Verify Rust toolchain
cargo --version                 # Should show nightly toolchain
```

_WHY: This project requires Rust nightly toolchain for Edition 2024 features; all tools are managed by the Nix flake to ensure consistent versions across environments._

### Build Commands

```bash
# Rust builds
cargo build
cargo build --release
cargo build -p novelsaga-cli

# xtask automation (via cargo alias)
cargo xtask build-js            # Build JS bridges
cargo xtask cli                 # Build CLI binary
cargo xtask build-all           # Build everything

# Nix builds
nix build
nix build .#bundle
```

_WHY: xtask provides cross-platform build orchestration; the cargo alias in `.cargo/config.toml` makes it callable as `cargo xtask`._

### Test Commands

```bash
cargo test                      # All tests
cargo test -p novelsaga-cli     # Package-specific
cargo test <name>               # Single test by name
cargo test -- --nocapture       # With output

# E2E tests
cargo xtask e2e
cargo xtask lsp-e2e
```

_WHY: Tests are colocated in source files (not `tests/` folders) to keep context close; E2E tests verify full CLI workflows._

### TDD Workflow

Follow test-driven development:

1. **Red**: Write failing test first
2. **Green**: Write minimal code to make test pass
3. **Refactor**: Clean up while keeping tests green

_WHY: TDD ensures code is testable by design, catches bugs early, and documents behavior through tests._

### Large Changes Workflow

For extensive refactors or multi-file changes, use git-worktree:

```bash
# Create a new worktree for your feature
git worktree add ../nsaga-feature-branch feature-branch-name
cd ../nsaga-feature-branch

# Work on your changes...
# If something goes wrong, the original worktree is untouched

# Clean up when done
git worktree remove ../nsaga-feature-branch
```

_WHY: Worktrees provide isolation for risky changes; if a refactor goes wrong, your original working directory remains intact._

### Lint & Format

```bash
# Rust
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo fmt

# TypeScript
pnpm exec eslint .
pnpm exec prettier --write .

# Unified (runs all formatters)
treefmt
```

_WHY: Pre-commit hooks (defined in `flake.nix`) automatically run clippy, eslint, and treefmt; manual runs should match hook behavior._

---

## Code Style Conventions

### Rust

**Formatting** (`rustfmt.toml`):

- 2-space indentation, 120 char width
- Import grouping: `StdExternalCrate` (std → external → crate)
- Vertical trailing commas only

**Import Order**:

```rust
use std::sync::Arc;                           // 1. std
use novelsaga_core::state::Initializer;       // 2. external crates
use crate::{args::Cli, bridge::BridgeManager}; // 3. crate
```

_WHY: Consistent ordering reduces merge conflicts and makes dependencies obvious._

**Naming**:
| Type | Convention | Example |
|------|-----------|---------|
| Functions/variables | `snake_case` | `handle_command`, `bridge_manager` |
| Structs/Enums/Traits | `PascalCase` | `BridgeManager`, `MetadataQuery` |
| Constants | `SCREAMING_SNAKE_CASE` | `PARSE_ERROR` |
| Modules | `snake_case` | `mod metadata;` |

**Error Handling**:

```rust
// Libraries: structured errors with thiserror
#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("JSON parse error: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("RPC error: code={code}, message={message}")]
    RpcError { code: i32, message: String },
}

// Applications: anyhow for propagation
use anyhow::Result;
fn main() -> Result<()> { ... }
```

_WHY: `thiserror` provides structured errors for API consumers; `anyhow` is ergonomic for CLI error propagation._

**Testing** (in-file only):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature() {
        assert_eq!(result, expected);
    }
}
```

_WHY: Colocated tests reduce navigation overhead and keep test context close to implementation._

### TypeScript

**Modules**: ESM only (`"type": "module"`), target ESNext, strict mode.

**Imports**:

```typescript
import type { BridgeConfig } from '@nsaga/bridge-core' // type imports first
import { BridgeServer } from '@nsaga/bridge-core' // value imports
import { localUtil } from './utils.js' // relative with .js
```

_WHY: Explicit type imports help bundlers tree-shake; relative imports need `.js` extension for ESM compatibility._

**Naming**:
| Type | Convention | Example |
|------|-----------|---------|
| Variables/functions | `camelCase` | `createBridgeServer` |
| Classes/Interfaces | `PascalCase` | `BridgeServer`, `ErrorHandler` |

---

## Technology-Specific Rules

### Metadata Storage

- **Canonical Path**: `<workspace>/.cache/novelsaga/sled`
- **Database**: Sled (embedded KV)
- **Shared Access**: CLI and LSP resolve the same canonical path

_WHY: Shared database allows CLI and LSP to see consistent state without synchronization overhead._

### JSON-RPC Bridge Protocol

Format: `"service.method"`

```typescript
// Request
{ "jsonrpc": "2.0", "method": "config.get", "params": { "key": "theme" }, "id": 1 }
// Response
{ "jsonrpc": "2.0", "result": { "theme": "dark" }, "id": 1 }
```

Use `createBridgeServer()` factory for bridge creation.

_WHY: Namespace prefix prevents method collisions when multiple services run in one process._

### TypeScript Integration

Export Rust types via `ts-rs`:

```rust
#[derive(TS)]
#[ts(export, export_to = "_config.ts")]
pub struct Config {
    pub name: String,
}
```

_WHY: `ts-rs` generates TypeScript bindings at compile time, keeping Rust and TS types in sync._

---

## Git Hooks (Automated)

Defined in `flake.nix`, run on every commit:

1. **commitizen** — Conventional commit format enforcement
2. **clippy** — `cargo clippy --all-targets --all-features --workspace -- -D warnings`
3. **eslint** — Lint `.ts/.cts/.mts` files
4. **treefmt** — Unified formatting (rustfmt, alejandra, taplo, shellcheck)

_WHY: Automated hooks ensure all commits meet quality standards without manual intervention._

---

## Summary Checklist

Before committing, verify:

- [ ] `cargo clippy` passes (or will pass via hook)
- [ ] Tests are in the source file, not separate `.test.rs`
- [ ] TypeScript imports use `node:` prefix for built-ins
- [ ] Bridges log to stderr, not stdout
- [ ] State is initialized with `Initializer::init()` before use
- [ ] Commit includes AI attribution trailer if AI-assisted

---

## AI-Assisted Commit Guidelines

**Key Points:**

- Always disclose AI assistance in commit messages
- Use `Co-authored-by:` trailer for AI collaboration attribution
- Keep attribution format consistent and traceable
- Human remains responsible for the committed code

### Attribution Format

Add at the end of commit message (before any sign-offs):

```
Co-authored-by: <AI-Tool-Name> <ai@example.com>
```

**Examples:**

```
Co-authored-by: Claude <noreply@anthropic.com>
Co-authored-by: Sisyphus <clio-agent@sisyphuslabs.ai>
```

### Full Template

```
<type>: <short summary>

<body - what and why>

Co-authored-by: <AI-Name> <ai-email>
```

**Example:**

```
feat: add metadata caching for LSP

Implement in-memory cache for entity metadata to reduce
disk I/O during LSP operations. Cache invalidation uses
file modification timestamps.

Co-authored-by: Sisyphus <clio-agent@sisyphuslabs.ai>
```

### Why This Matters

_Tracking AI assistance provides:_

- **Transparency**: Contributors know when code was AI-assisted
- **Traceability**: Git history shows collaboration patterns
- **Accountability**: Human remains the commit author responsible for code quality
- **Compliance**: Prepares for emerging open-source AI contribution policies (see Linux Kernel RFC on AI coding assistants)

### Reference

- Linux Kernel RFC: `Co-developed-by:` trailer for AI attribution
- Git trailers: Any `Key: Value` format is valid (Git 2.32+)
- GitHub: `Co-authored-by:` displayed as co-author in UI

Key workspace facts:

- **Edition**: Rust 2024 (nightly toolchain)
- **Indent**: 2 spaces (not 4)
- **Tests**: Colocated in `#[cfg(test)]` modules
- **Package manager**: pnpm (never npm in subprojects)
- **Development**: TDD workflow, git-worktree for large changes

<!-- OCR:START -->

## Open Code Review Instructions

These instructions are for AI assistants handling code review in this project.

Always open `.ocr/skills/SKILL.md` when the request:

- Asks for code review, PR review, or feedback on changes
- Mentions "review my code" or similar phrases
- Wants multi-perspective analysis of code quality
- Asks to map, organize, or navigate a large changeset

Use `.ocr/skills/SKILL.md` to learn:

- How to run the 8-phase review workflow
- How to generate a Code Review Map for large changesets
- Available reviewer personas and their focus areas
- Session management and output format

Keep this managed block so 'ocr init' can refresh the instructions.

<!-- OCR:END -->
