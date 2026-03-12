# NOVELSAGA PROJECT KNOWLEDGE BASE

## OVERVIEW

**NovelSaga** - Multi-Editor Novel Editing System Backend (Rust + TypeScript)

- **LSP Server**: For editors with LSP support (VSCode, Neovim)
- **C Library**: Via Diplomat FFI for iOS, Android, Web (.so/WASM)
- **JS Bridges**: JSON-RPC 2.0 communication between Rust CLI and Node.js/Bun/Deno

## TECHNOLOGY STACK

- **Rust**: Core logic, LSP server, and FFI bindings.
- **TypeScript**: Bridge implementations and client-side tooling.
- **Diplomat FFI**: Generates bindings for multiple languages from Rust source.
- **JSON-RPC 2.0**: Protocol for CLI-to-Bridge communication.
- **Nix**: Reproducible development environments and builds.

## PROJECT STRUCTURE

```
nsaga/
├── projects/
│   ├── core/              # Rust core library: state, config, and domain logic
│   ├── cli/               # Rust LSP server & CLI: bridge management and discovery
│   └── cli-js-bridges/    # TypeScript bridges: filesystem, git, and external services
├── xtask/                 # Build automation and development workflows
├── docs/                  # Project documentation
└── out/                   # Compiled outputs and build artifacts
```

## METADATA STORAGE CONTRACT

NovelSaga maintains a persistent metadata store for indexing and state management.

- **Canonical Path**: `<workspace>/.cache/novelsaga/sled`
- **Database**: Sled (embedded key-value store)
- **Shared Access**: The CLI and LSP server share the same database by resolving the canonical path relative to the workspace root.
- **Fallback Policy**: The resolver uses a pure context-derived fallback. It does not rely on specific markers like `metadata/`.

## QUICK COMMANDS

```bash
# Development Environment (REQUIRED)
direnv allow                          # Load Nix environment

# Build
pnpm install                          # Install JS deps (root only)
xtask build-js                        # Build JS bridges and generate types
xtask cli                             # Build CLI for the current platform
xtask build-all                       # Build the entire project
cargo build                           # Standard Rust build

# Test
cargo test                            # Run all Rust tests
cargo test -p novelsaga-cli           # Run CLI tests only
cargo test <name> -- --ignored        # Run specific ignored tests
xtask e2e                             # Execute end-to-end tests

# Lint & Format
cargo clippy --all-targets            # Lint Rust code
pnpm exec eslint .                    # Lint JS/TS code
pnpm exec prettier --write .          # Format JS/TS code

# Nix Build
nix build                             # Build CLI via Nix
nix build .#bundle                    # Build all platform bundles
```

## CONVENTIONS

### Rust

- **Edition 2024**: Nightly features allowed (e.g., `#![feature(mpmc_channel)]`).
- **In-file Tests**: Modules marked with `#[cfg(test)]` stay in the same file as source.
- **Type Exports**: Use `#[derive(TS)]` with `#[ts(export, export_to = "_config.ts")]` for TS integration.
- **State Initialization**: Call `Initializer::init(feature)` before attempting `Initializer::get()`.

### TypeScript/JavaScript

- **Runtime**: Node.js ≥20 (Native TS support preferred for newer versions).
- **Modules**: ESM only (`"type": "module"`).
- **Package Management**: pnpm workspaces. Never install dependencies in subprojects.
- **Bridge Pattern**: Use `createBridgeServer()` factories for service registration.
- **Routing**: Use `"service.method"` format for JSON-RPC calls.

### Git & Commits

- **Style**: Conventional commits (`feat:`, `fix:`, `refactor:`).
- **Hooks**: Automated checks via clippy and treefmt.

## ANTI-PATTERNS

| Category     | Forbidden                                    | Do Instead                                      |
| :----------- | :------------------------------------------- | :---------------------------------------------- |
| Dependencies | `npm install` in subprojects                 | `pnpm install` at workspace root                |
| Types        | Manual edits to `_config.ts`                 | Extend types in separate files                  |
| State        | Accessing `Initializer` before `init`        | Ensure `init()` is called during startup        |
| Bridges      | Relative imports crossing project boundaries | Use `@nsaga/` workspace aliases                 |
| Logs         | Printing to `stdout` in bridges              | Use `stderr` for logs; `stdout` is for JSON-RPC |
| Tests        | Creating separate `.test.rs` files           | Keep tests in the source file                   |

## VSCODE DEVELOPMENT

### Recommended Setup

1. **Install Extensions**: Accept the workspace recommended extensions.
2. **Environment**: Run `direnv allow` in your terminal to sync the Nix environment.
3. **Status Bar**: Confirm `rust-analyzer` and `ESLint` are active.

### Keyboard Shortcuts

- **Go to Definition**: `F12`
- **Find All References**: `Shift + F12`
- **Rename Symbol**: `F2`
- **Quick Fix**: `Ctrl + .` (or `Cmd + .`)
- **Command Palette**: `Ctrl + Shift + P`
- **Search Files**: `Ctrl + P`
