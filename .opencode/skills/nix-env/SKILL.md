---
name: nix-env
description: Nix development environment setup and management for NovelSaga
---

# Nix Development Environment

## Quick Reference

- **Setup**: `direnv allow` (loads environment automatically via `.envrc`)
- **Verify Tools**: `which cargo`, `which node` (should point to Nix store)
- **Reload**: `direnv reload` (after flake.nix changes)
- **Check Status**: `direnv status` (view environment info)

## Development Environment Setup

### First-Time Setup

```bash
# 1. Load environment (required on fresh clone)
direnv allow

# 2. Verify toolchain is available
which cargo          # Should output Nix store path (/nix/store/...)
which node           # Should output Nix store path (/nix/store/...)
which pnpm           # Should output Nix store path (/nix/store/...)

# 3. Verify Rust toolchain
rustc --version      # Should show nightly channel
cargo --version
```

### Environment Components

The NovelSaga development environment provides:

| Tool/Language       | Version     | Notes                                        |
| ------------------- | ----------- | -------------------------------------------- |
| **Rust**            | nightly     | With targets: x86_64, aarch64, Android, WASM |
| **Node.js**         | 24.x (slim) | Minimal variant for faster loads             |
| **pnpm**            | latest      | Package manager (enabled via devenv)         |
| **Nix**             | latest      | Via `nil` LSP                                |
| **Git Hooks**       | devenv      | commitizen, clippy, eslint, treefmt          |
| **Cross-compilers** | multiple    | For Linux, Windows, Android, macOS targets   |

### Available Scripts

```bash
xtask <command>      # Run build tasks (shorthand for cargo run --package xtask)
# Examples:
xtask build-js       # Build JS bridges
xtask cli            # Build CLI for current platform
xtask build-all      # Build everything
```

### Rust Components Loaded

When you run `direnv allow`, these Rust components are available:

- `rustc` - Rust compiler
- `cargo` - Package manager
- `clippy` - Linter
- `rustfmt` - Code formatter
- `rust-analyzer` - IDE support
- `miri` - Undefined behavior detector
- `rust-std` - Standard library

### Cross-Compilation Targets

The environment includes pre-configured toolchains for:

```
- x86_64-unknown-linux-gnu      # Current Linux (most common)
- aarch64-unknown-linux-gnu     # ARM64 Linux
- aarch64-apple-darwin          # Apple Silicon macOS
- x86_64-apple-darwin           # Intel macOS
- aarch64-pc-windows-gnullvm    # ARM64 Windows
- x86_64-pc-windows-gnu         # Intel Windows
- wasm32-unknown-unknown         # WebAssembly
- aarch64-linux-android          # Android ARM64
```

## Environment File Watching

The `.envrc` configuration watches these files for changes:

- `flake.nix` - Main environment definition
- `flake.lock` - Locked dependency versions
- `.vscode/settings.nix` - VS Code settings generation

If you modify these files, direnv will automatically reload the environment. You can also manually reload with:

```bash
direnv reload
```

## Git Hooks

The environment includes pre-commit hooks:

| Hook           | Purpose                                         |
| -------------- | ----------------------------------------------- |
| **commitizen** | Enforce conventional commit format              |
| **clippy**     | Run Rust linter on staged code                  |
| **eslint**     | Run JavaScript/TypeScript linter                |
| **treefmt**    | Format code (Nix, Rust, JavaScript, Bash, TOML) |

These run automatically on `git commit`. To run manually:

```bash
# Format all code
treefmt

# Lint Rust
cargo clippy --all-targets --all-features --workspace

# Lint TypeScript
pnpm exec eslint .
```

## Cross-Compilation Environment Variables

The environment sets up cross-compilation variables for multiple platforms:

```bash
# Linux targets
CC_x86_64_unknown_linux_gnu
CXX_x86_64_unknown_linux_gnu
AR_x86_64_unknown_linux_gnu

# Windows targets (via MinGW)
CC_x86_64_pc_windows_gnu
CXX_x86_64_pc_windows_gnu

# Android target
CC_aarch64_linux_android
CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER
CARGO_TARGET_AARCH64_LINUX_ANDROID_AR

# Bootstrap flag for nightly features
RUSTC_BOOTSTRAP=1
```

These are set automatically and do not require manual configuration.

## Troubleshooting

### Environment Not Loading

**Problem**: `direnv: error: command not found: use`

**Solution**:

1. Ensure Nix is installed: `nix --version`
2. Enable nix-direnv: Check that `.envrc` exists and is executable
3. Reload: `direnv allow` and `direnv reload`

---

### Tools Not Found in PATH

**Problem**: `which cargo` returns no result or finds system cargo

**Solution**:

```bash
# Force reload environment
direnv unload
direnv allow
direnv reload

# Verify Nix environment is active
direnv status

# Check if in Nix shell
echo $IN_NIX_SHELL
```

---

### Rust-analyzer Not Working in VS Code

**Problem**: rust-analyzer shows "initializing..." indefinitely or reports missing Rust toolchain

**Solution**:

1. Run `direnv allow` in terminal
2. Reload VS Code: `Ctrl+Shift+P` → "Developer: Reload Window"
3. Verify environment: Open integrated terminal and run `which rustc`
4. Check rust-analyzer status: Click "Rust Analyzer" in status bar

---

### Type Generation Out of Date

**Problem**: TypeScript types in `_config.ts` don't match Rust code

**Solution**:

```bash
# Regenerate types from Rust
xtask build-js

# This will update: projects/cli-js-bridges/config-bridge/src/types/_config.ts
```

---

### Nightly Features Error

**Problem**: Compilation fails with "this feature is currently unstable"

**Solution**:

- The environment should have `RUSTC_BOOTSTRAP=1` set
- Verify: `echo $RUSTC_BOOTSTRAP` (should output `1`)
- If not set, run `direnv reload`

---

### Lock File Conflicts After Updates

**Problem**: `flake.lock` has conflicts or refuses to update

**Solution**:

```bash
# Update specific input
nix flake update <input-name>

# Example: Update nixpkgs
nix flake update nixpkgs

# Update everything
nix flake update
```

---

### Performance: Slow direnv Loading

**Problem**: `direnv allow` or reload takes >30 seconds

**Solution**:

- First load is normal (downloads toolchain from cache)
- Subsequent loads should be instant
- Check cache: `nix-store --verify --check-contents`
- Force GC: `nix-collect-garbage`

---

### VSCode Settings Not Applying

**Problem**: VS Code settings don't match `flake.nix` configuration

**Solution**:

- The `.vscode/settings.json` is generated from `.vscode/settings.nix`
- Modify `settings.nix`, not `settings.json` (it gets regenerated)
- Reload environment: `direnv reload`

## Anti-Patterns

| Don't                                       | Do Instead                                                   |
| ------------------------------------------- | ------------------------------------------------------------ |
| Install tools globally with `nix-env -i`    | Use `devenv` (loads from `flake.nix`)                        |
| Commit `flake.lock` changes without testing | Run `nix build` first to verify                              |
| Use system Node/Rust in this project        | Run `direnv allow` to load Nix-managed versions              |
| Modify `.vscode/settings.json` directly     | Edit `.vscode/settings.nix` instead (JSON is auto-generated) |
| Run `npm install` in subprojects            | Use `pnpm install` at root only                              |
| Skip `direnv allow` on fresh clone          | Environment won't load without explicit allow                |
| Hard-code tool paths                        | Reference tools by name (path is set by direnv)              |

## When to Use

Load this skill when:

- Setting up development environment for the first time
- Environment tools are not found in PATH
- Rust toolchain or dependencies behave unexpectedly
- VS Code or LSP not recognizing Rust/TypeScript files
- Need to update Nix inputs or regenerate type definitions
- Troubleshooting cross-compilation or native build issues
- Configuring new shell or terminal environment

**Related Skills**:

- **nix-build**: Use when building projects or updating hashes
- **rust-dev**: Use when modifying Rust code
- **ts-bridge**: Use when modifying TypeScript bridges
