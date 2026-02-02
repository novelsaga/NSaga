# NOVELSAGA PROJECT KNOWLEDGE BASE

## OVERVIEW

**NovelSaga** - Multi-Editor Novel Editing System Backend (Rust + TypeScript)

- **LSP Server**: For editors with LSP support (VSCode, Neovim)
- **C Library**: Via Diplomat FFI for iOS, Android, Web (.so/WASM)
- **JS Bridges**: JSON-RPC 2.0 communication between Rust CLI and Node.js/Bun/Deno

## STRUCTURE

```
nsaga/
├── projects/
│   ├── core/              # Rust core library - see projects/core/AGENTS.md
│   ├── cli/               # Rust LSP server & CLI - see projects/cli/AGENTS.md
│   └── cli-js-bridges/    # TypeScript bridges - see projects/cli-js-bridges/AGENTS.md
├── xtask/                 # Build automation (Rust)
├── docs/                  # Documentation
├── .opencode/             # OpenCode agent config, commands, progress tracking
└── out/                   # Build outputs
```

## WHERE TO LOOK

| Task            | Location                    | Notes                             |
| --------------- | --------------------------- | --------------------------------- |
| Rust core logic | `projects/core/src/`        | Config, state management          |
| CLI/LSP server  | `projects/cli/src/`         | Bridge manager, runtime discovery |
| JS bridges      | `projects/cli-js-bridges/`  | JSON-RPC services                 |
| Build tasks     | `xtask/src/tasks/`          | build-js, cli, wasm, so           |
| Config types    | `projects/core/src/config/` | `ts-rs` exports to TypeScript     |
| Tests (Rust)    | Same file as source         | `#[cfg(test)]` modules            |

## QUICK COMMANDS

```bash
# Development Environment (REQUIRED)
direnv allow                          # Load Nix environment

# Build
pnpm install                          # Install JS deps (root only!)
./xtask.sh build-js                   # Build JS bridges
./xtask.sh cli                        # Build CLI (current platform)
./xtask.sh build-all                  # Build everything
cargo build                           # Rust build

# Test
cargo test                            # All Rust tests
cargo test -p novelsaga-cli           # CLI tests only
cargo test <test_name> -- --ignored   # Run ignored integration tests
./xtask.sh e2e                        # End-to-end tests

# Lint & Format
cargo clippy --all-targets            # Rust linting
pnpm exec eslint .                    # JS/TS linting
pnpm exec prettier --write .          # Format JS/TS

# Nix Build
nix build                             # Build CLI via Nix
nix build .#bundle                    # Build all platforms
```

## CONVENTIONS

### Rust

- **Edition 2024**, nightly features (`#![feature(mpmc_channel)]`)
- Tests in same file using `#[cfg(test)]` modules
- Type exports to TS: `#[derive(TS)]` + `#[ts(export, export_to = "_config.ts")]`
- State init: `Initializer::init(feature)` before `Initializer::get()`

### TypeScript/JavaScript

- **Node.js ≥20**, ESM modules (`"type": "module"`)
- **pnpm workspace** - never install deps at subproject level
- Bridge pattern: `createBridgeServer()` factory + Service registration
- JSON-RPC routing: `"service.method"` format

### Git & Commits

- Commits: Conventional commits (`feat:`, `fix:`, `refactor:`)
- Pre-commit hooks: commitizen, clippy, treefmt

## ANTI-PATTERNS

| Category     | Forbidden                         | Do Instead                       |
| ------------ | --------------------------------- | -------------------------------- |
| Dependencies | `npm install` in subproject       | `pnpm install` at root           |
| Types        | Modify `_config.ts`               | Extend in separate files         |
| State        | `Initializer::get()` without init | Call `init()` first              |
| Bridges      | Relative imports outside rootDir  | Use `@nsaga/bridge-*`            |
| Logs         | stdout in bridges                 | stderr for logs, stdout for JSON |
| Tests        | Separate test files               | Same file `#[cfg(test)]`         |

## AI AGENT GUIDELINES

### Communication

1. **Seek Confirmation**: Ask before significant changes
2. **Interactive**: Stop at ambiguity, confirm architecture decisions
3. **Incremental**: Break complex tasks into steps with checkpoints

### Documentation

- AI-generated docs: `.opencode/docs/` (progress tracking)
- After refactoring: ALWAYS update related docs
- Never scatter docs across project

### Common Pitfalls

- Shell aliases: `ls→eza`, `cat→bat` - use tool functions instead
- Node version: Native TS requires ≥23.6
- Config search: Upward from current dir to workspace root

## PROGRESS TRACKING

- **Current Status**: See `.opencode/docs/NEXT_STEPS.md`
- **Completed Tasks**: See `.opencode/docs/COMPLETED_TASKS.md`
- **Backlog**: See `.opencode/docs/BACKLOG.md`

## VSCODE DEVELOPMENT

### Recommended Setup

1. **Install extensions**: Open workspace → Accept recommended extensions prompt
2. **Load environment**: `direnv allow` in terminal (critical for Nix)
3. **Verify**: Check rust-analyzer and ESLint are active in status bar

### Extension Categories

| Category       | Extensions                              | Purpose                    |
| -------------- | --------------------------------------- | -------------------------- |
| **Rust**       | rust-analyzer, Even Better TOML, crates | Core Rust development      |
| **TypeScript** | ESLint, Prettier, Error Lens            | JS/TS linting & formatting |
| **Nix**        | direnv, Nix IDE                         | Environment management     |
| **AI**         | Continue.dev, GitHub Copilot            | AI-assisted coding         |
| **Workspace**  | Monorepo Workspace, GitLens             | Navigation & git           |

### Keyboard Shortcuts (Productivity)

| Action              | macOS    | Windows/Linux  | Notes                 |
| ------------------- | -------- | -------------- | --------------------- |
| Go to Definition    | `F12`    | `F12`          | Works for Rust & TS   |
| Find All References | `⇧F12`   | `Shift+F12`    | Cross-language        |
| Rename Symbol       | `F2`     | `F2`           | LSP-powered refactor  |
| Quick Fix           | `⌘.`     | `Ctrl+.`       | Clippy & ESLint fixes |
| Command Palette     | `⌘⇧P`    | `Ctrl+Shift+P` | All commands          |
| Integrated Terminal | `` ⌃` `` | `` Ctrl+` ``   | Run commands          |
| Search Files        | `⌘P`     | `Ctrl+P`       | Quick file open       |
| Search Symbols      | `⌘T`     | `Ctrl+T`       | Workspace symbols     |

### AI Assistant Integration

#### GitHub Copilot

- `⌘I` / `Ctrl+I` - Inline chat
- Follows `.github/copilot-instructions.md` automatically
- Best for: Quick completions, simple refactors

#### Continue.dev

- `⌘L` / `Ctrl+L` - Open Continue chat
- `⌘⇧L` / `Ctrl+Shift+L` - Add selection to chat
- `@codebase` - Search entire codebase
- `@file` - Reference specific file
- Best for: Complex tasks, multi-file changes

### Troubleshooting

| Issue                     | Solution                                                         |
| ------------------------- | ---------------------------------------------------------------- |
| rust-analyzer not loading | Run `direnv allow`, then reload window (`⌘⇧P` → "Reload Window") |
| ESLint not working        | Verify `pnpm install` completed at root                          |
| Types seem outdated       | Run `./xtask.sh build-js` to regenerate                          |
| Nix env not active        | Check terminal shows Nix-provided tools                          |
| Extensions not installed  | `⌘⇧P` → "Extensions: Show Recommended Extensions"                |

## MODULE GUIDES

For detailed module-specific information, see:

- [`projects/core/AGENTS.md`](projects/core/AGENTS.md) - Core library details
- [`projects/cli/AGENTS.md`](projects/cli/AGENTS.md) - CLI & LSP server details
- [`projects/cli-js-bridges/AGENTS.md`](projects/cli-js-bridges/AGENTS.md) - JS bridge details
