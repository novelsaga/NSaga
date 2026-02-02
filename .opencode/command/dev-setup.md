---
description: Initialize development environment for new contributors
argument-hint: []
---

<command-instruction>
Set up the NovelSaga development environment from scratch.

## Prerequisites

- Nix package manager installed
- direnv installed and hooked into shell

## Steps

1. **Load Nix environment**:

   ```bash
   direnv allow
   ```

2. **Install JavaScript dependencies**:

   ```bash
   pnpm install
   ```

3. **Build JS bridges**:

   ```bash
   ./xtask.sh build-js
   ```

4. **Build Rust CLI**:

   ```bash
   cargo build
   ```

5. **Run tests to verify**:
   ```bash
   cargo test
   ```

## Verification Checklist

After setup, verify:

- [ ] `cargo --version` shows nightly toolchain
- [ ] `node --version` shows v20+
- [ ] `pnpm --version` shows v9+
- [ ] `./xtask.sh build-js` completes without errors
- [ ] `cargo test` passes

## VSCode Setup

1. Open workspace in VSCode
2. Accept recommended extensions when prompted
3. Wait for rust-analyzer to index (may take 1-2 minutes)
4. Verify ESLint is active (check status bar)

## Troubleshooting

| Issue                | Solution                                     |
| -------------------- | -------------------------------------------- |
| direnv not loading   | Add `eval "$(direnv hook bash)"` to shell rc |
| rust-analyzer errors | Reload VSCode window after direnv allow      |
| pnpm not found       | Ensure Nix environment loaded                |

</command-instruction>

<current-context>
<environment_status>
!`echo "Nix: $(nix --version 2>/dev/null || echo 'not found')"; echo "Cargo: $(cargo --version 2>/dev/null || echo 'not found')"; echo "Node: $(node --version 2>/dev/null || echo 'not found')"; echo "pnpm: $(pnpm --version 2>/dev/null || echo 'not found')"`
</environment_status>
<direnv_status>
!`direnv status 2>/dev/null | head -5 || echo "direnv not active"`
</direnv_status>
</current-context>
