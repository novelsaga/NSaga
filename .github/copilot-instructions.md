# NovelSaga Development Guide (GitHub Copilot)

> **Note**: This is a simplified reference for GitHub Copilot. For comprehensive documentation, see [`AGENTS.md`](../AGENTS.md) at project root.

## Quick Reference

**NovelSaga** - Multi-Editor Novel Editing System Backend (Rust + TypeScript)

### Project Structure

```
nsaga/
├── projects/
│   ├── core/              # Rust core library
│   ├── cli/               # Rust LSP server & CLI
│   └── cli-js-bridges/    # TypeScript JSON-RPC bridges
├── xtask/                 # Build automation
├── .sisyphus/             # OpenCode work plans and notepads
└── .opencode/             # OpenCode configuration (skills, commands)
```

### Essential Commands

```bash
# Development environment (REQUIRED)
direnv allow                # Load Nix environment

# Build
pnpm install               # Install JS deps (root only!)
./xtask.sh build-js        # Build JS bridges
cargo build                # Rust build

# Test
cargo test                 # All Rust tests
./xtask.sh e2e             # End-to-end tests

# Lint
cargo clippy --all-targets # Rust linting
pnpm exec eslint .         # JS/TS linting
```

### Key Conventions

- **Rust**: Edition 2024, nightly, tests in same file (`#[cfg(test)]`)
- **TypeScript**: Node.js ≥20, ESM, pnpm workspace
- **Dependencies**: ALWAYS install at root level
- **Git**: Conventional commits (`feat:`, `fix:`, etc.)

### Anti-Patterns

| Don't                             | Do Instead                       |
| --------------------------------- | -------------------------------- |
| `npm install` in subproject       | `pnpm install` at root           |
| Modify `_config.ts`               | Extend in separate files         |
| `Initializer::get()` without init | Call `init()` first              |
| stdout for logs in bridges        | stderr for logs, stdout for JSON |

## Detailed Documentation

- **Root Guide**: [`AGENTS.md`](../AGENTS.md)
- **Core Module**: [`projects/core/AGENTS.md`](../projects/core/AGENTS.md)
- **CLI Module**: [`projects/cli/AGENTS.md`](../projects/cli/AGENTS.md)
- **JS Bridges**: [`projects/cli-js-bridges/AGENTS.md`](../projects/cli-js-bridges/AGENTS.md)
- **Progress Tracking**: [`.sisyphus/roadmaps/`](../.sisyphus/roadmaps/) and [`.sisyphus/plans/`](../.sisyphus/plans/)

## AI Agent Guidelines

1. **Seek Confirmation**: Ask before significant changes
2. **Interactive**: Stop at ambiguity, confirm architecture decisions
3. **Incremental**: Break complex tasks into steps
