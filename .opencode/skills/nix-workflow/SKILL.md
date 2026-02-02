---
name: nix-workflow
description: Nix development environment and build system for NovelSaga - use for Nix configuration or reproducible builds
---

# Nix Workflow

## Quick Reference

- **Development**: `direnv allow` loads environment automatically
- **Build CLI**: `nix build` or `nix build .#cli`
- **Build All**: `nix build .#bundle`

## Development Environment

```bash
# First time setup
direnv allow

# Verify environment
which cargo          # Should point to Nix store
which node           # Should point to Nix store
```

## Build Targets

| Target      | Command              | Output                 |
| ----------- | -------------------- | ---------------------- |
| CLI only    | `nix build`          | `result/bin/novelsaga` |
| Full bundle | `nix build .#bundle` | CLI + WASM + .so       |

## Hash Update Process

When `pnpm-lock.yaml` changes:

1. Set placeholder in `packages.nix`:

   ```nix
   hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
   ```

2. Get correct hash:

   ```bash
   nix build .#bundle 2>&1 | grep "got:"
   ```

3. Update `packages.nix` with new hash

## Troubleshooting

| Issue                     | Solution                          |
| ------------------------- | --------------------------------- |
| Hash mismatch             | Update pnpmDeps hash              |
| Nightly features error    | Ensure fenix overlay active       |
| rust-analyzer not working | Run `direnv allow`, reload VSCode |

## When to Use

Load this skill when:

- Modifying `flake.nix`, `packages.nix`
- Running Nix builds
- Troubleshooting environment issues
