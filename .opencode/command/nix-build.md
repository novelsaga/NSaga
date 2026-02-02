---
description: Build NovelSaga via Nix (CLI or full bundle)
argument-hint: [cli|bundle]
---

<command-instruction>
Build NovelSaga using Nix for reproducible builds.

## Available Packages

### CLI Only (Default)

```bash
nix build
# or
nix build .#cli
```

**Output**: `result/bin/novelsaga` + JS assets

### Full Bundle (All Platforms)

```bash
nix build .#bundle
```

**Output**: CLI for all platforms + WASM + shared libraries

## Build Process

### CLI Build (`nix build`)

1. Nix fetches pnpm dependencies (offline, using hash)
2. `pnpmConfigHook` installs to `node_modules`
3. JS bridges built via `node build.mts`
4. Rust CLI built with nightly toolchain
5. Assets copied to `$out/share/novelsaga/assets/js/dist/`

### Bundle Build (`nix build .#bundle`)

1. Same JS dependency setup
2. All Rust targets built: CLI (3 platforms), WASM, shared libs
3. Output organized in `$out/share/novelsaga/{cli,wasm,so}/`

## Updating pnpmDeps Hash

When `pnpm-lock.yaml` changes:

1. Set placeholder hash in `packages.nix`:

   ```nix
   hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
   ```

2. Run build to get correct hash:

   ```bash
   nix build .#bundle 2>&1 | grep "got:"
   ```

3. Update `packages.nix` with the new hash

## Verifying Build

```bash
# Check CLI works
./result/bin/novelsaga --help

# Check assets included
ls ./result/share/novelsaga/assets/js/dist/

# For bundle, check all outputs
ls ./result/share/novelsaga/
```

## Troubleshooting

- **Hash mismatch**: Update pnpmDeps hash (see above)
- **Nightly features**: Ensure `fenix` overlay provides nightly Rust
- **Network error**: All fetches should be pre-cached; check flake inputs
  </command-instruction>

<current-context>
<nix_status>
!`nix --version 2>/dev/null || echo "Nix not available"`
</nix_status>
<current_result>
!`ls -la result 2>/dev/null || echo "No previous build result"`
</current_result>
</current-context>
