---
description: Run end-to-end tests for config loading
argument-hint: [--runtime <node|bun|deno>]
---

<command-instruction>
Run E2E tests for the NovelSaga CLI config loading system.

## Prerequisites

1. Ensure JS bridges are built:

   ```bash
   ./xtask.sh build-js
   ```

2. Ensure Rust CLI is compiled:
   ```bash
   cargo build -p novelsaga-cli
   ```

## Run Tests

**All E2E tests** (14 scenarios):

```bash
./xtask.sh e2e
```

**Or run specific test via cargo**:

```bash
cargo test -p novelsaga-cli test_e2e -- --ignored --nocapture
```

## Test Matrix

The E2E tests cover:

- **3 runtimes**: Node.js, Bun, Deno
- **4 config formats**: .mjs, .cjs, .ts, .mts (TS skipped if Node < 23.6)
- **2 additional tests**: Config priority, error handling

Total: 14 test scenarios

## Test Output

Tests create temporary config files and verify:

1. Runtime correctly spawns
2. Config bridge loads the file
3. JSON-RPC communication works
4. Config values are correctly parsed

## Troubleshooting

- **Node TS tests skipped**: Normal if Node.js < 23.6 (no native TS support)
- **Bun/Deno not found**: Install them or use `--runtime node`
- **Bridge spawn failed**: Run `./xtask.sh build-js` first
  </command-instruction>

<current-context>
<available_runtimes>
!`command -v node && node --version; command -v bun && bun --version; command -v deno && deno --version 2>/dev/null || echo "Some runtimes not available"`
</available_runtimes>
<js_bridges_status>
!`ls projects/cli/assets/js/dist/*.js 2>/dev/null || echo "JS bridges not built"`
</js_bridges_status>
</current-context>
