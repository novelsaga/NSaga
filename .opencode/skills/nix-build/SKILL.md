---
name: nix-build
description: Nix build system for reproducible builds and multi-platform compilation
---

# Nix Build System

## Quick Reference

| Command                       | Output                 | Use Case                         |
| ----------------------------- | ---------------------- | -------------------------------- |
| `nix build`                   | `result/bin/novelsaga` | Build CLI for current platform   |
| `nix build .#cli`             | `result/bin/novelsaga` | Same as above (explicit)         |
| `nix build .#bundle`          | CLI + WASM + .so       | Build all platforms (Linux only) |
| `nix build .#pnpmHashUpdater` | Hash extraction        | Compute pnpm dependencies hash   |

## Build Targets

### CLI Build (Default)

Default target builds the NovelSaga CLI for the current platform:

```bash
nix build
# or explicitly:
nix build .#cli
```

**Output**: `result/bin/novelsaga` (executable)

**Includes**:

- Rust CLI binary
- JavaScript bridges (compiled at build time)
- JS assets in `$out/share/novelsaga/assets/js/dist/`

### Full Bundle Build

Multi-platform bundle with CLI, WASM, and native libraries:

```bash
nix build .#bundle
```

**Available on**: `x86_64-linux` only (requires cargo-zigbuild)

**Output structure**:

```
$out/share/novelsaga/
├── cli/     # CLI binaries for all targets
├── wasm/    # WebAssembly artifacts
└── so/      # Native libraries (.so/.dylib/.dll)
```

**Targets included**:

- Linux: x86_64, aarch64
- Windows: x86_64, aarch64
- macOS: x86_64, aarch64
- Android: aarch64
- WebAssembly: wasm32-unknown-unknown

## Hash Update Procedure

When `pnpm-lock.yaml` changes, the pnpm dependencies hash must be updated.

### Automated (Recommended)

Use the GitHub Actions workflow (triggered on pnpm-lock.yaml changes):

1. Workflow computes hashes for all platforms (Linux x86_64/aarch64, macOS x86_64/aarch64)
2. Hashes are collected in `nix/hashes.json`
3. Changes auto-committed and pushed to main

### Manual Hash Update

For local development, to get the correct hash:

1. **Set placeholder in `nix/packages.nix`**:

   In the `pnpmHashUpdater` section, use:

   ```nix
   hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
   ```

2. **Build to reveal correct hash**:

   ```bash
   nix build .#pnpmHashUpdater --no-link 2>&1 | grep "got:"
   ```

   This will fail with output like:

   ```
   hash mismatch in fixed-output derivation '/nix/store/...-pnpm-hash-updater':
     wanted: sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
     got:    sha256-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=
   ```

3. **Update `nix/hashes.json`**:

   Copy the "got:" hash to `nix/hashes.json` for the appropriate system:

   ```json
   {
     "pnpmDeps": {
       "x86_64-linux": "sha256-XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX=",
       "aarch64-linux": "sha256-...",
       "x86_64-darwin": "sha256-...",
       "aarch64-darwin": "sha256-..."
     }
   }
   ```

4. **Verify**:

   ```bash
   nix build .#bundle  # Should succeed now
   ```

## Build Process Details

### JavaScript Bridge Compilation

Before Rust compilation, all JavaScript bridges are compiled:

- `bridge-core` (dependency of others)
- `bridge-nodejs`, `bridge-bun`, `bridge-deno` (runtime adapters)
- `config-bridge` (type definitions and configuration)

Build scripts are called via: `node build.mts` in each bridge directory

### Multi-Platform Compilation (Bundle)

The bundle build uses `cargo-zigbuild` for cross-compilation:

**Targets and toolchains**:

- `x86_64-unknown-linux-gnu` → gcc
- `aarch64-unknown-linux-gnu` → aarch64-unknown-linux-gnu-gcc
- `x86_64-pc-windows-gnu` → x86_64-w64-mingw32-gcc
- `aarch64-pc-windows-gnullvm` → aarch64-w64-mingw32-clang
- `aarch64-linux-android` → Android NDK toolchain
- `wasm32-unknown-unknown` → Rust nightly wasm target
- macOS targets: Built on native runners (GitHub Actions)

## Anti-Patterns

| Don't                                         | Do Instead                                                     |
| --------------------------------------------- | -------------------------------------------------------------- |
| Edit `nix/packages.nix` hash directly         | Use `nix build .#pnpmHashUpdater` and update `nix/hashes.json` |
| Use different pnpmDeps hash for each artifact | All artifacts share same hash from `hashes.json`               |
| Build bundle on non-Linux systems             | Use GitHub Actions or x86_64-linux for `.#bundle`              |
| Ignore platform-specific hash mismatches      | Update all system entries in `nix/hashes.json`                 |
| Rebuild entire workspace after hash change    | Hash is auto-fetched; just run `nix build` again               |

## Debugging

### Hash Mismatch Errors

If you see `hash mismatch in fixed-output derivation`:

1. Check `nix/hashes.json` has entry for your system
2. Run build again to reveal correct hash
3. Update JSON and retry

### Build Failures by Platform

**Linux x86_64/aarch64**: Use `nix build .#pnpmHashUpdater` then `.#bundle`

**macOS**: Only native builds supported; use GitHub Actions for cross-compilation

**Android**: Requires Android NDK; only available in `.#bundle` target

## When to Use

Load this skill when:

- Running `nix build` for any target
- Updating pnpm dependencies
- Building for multiple platforms
- Debugging hash mismatches
- Deploying via Nix to production
