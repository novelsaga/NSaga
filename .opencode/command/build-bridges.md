---
description: Build JS bridges (incremental or forced)
argument-hint: [--force]
---

<command-instruction>
Build all JavaScript bridges for the NovelSaga CLI.

## Steps

1. Check current JS bridge build status:

   ```bash
   ls -la projects/cli-js-bridges/*/dist/ 2>/dev/null || echo "No dist directories found"
   ```

2. Run the build:
   - **Incremental build** (default): `./xtask.sh build-js`
   - **Force rebuild all**: `./xtask.sh build-js --force`

3. Verify outputs exist:
   ```bash
   ls projects/cli/assets/js/dist/
   ```

## Build Order

The bridges are built in dependency order:

1. `build-tool` - Build utilities (dependency of all bridges)
2. `bridge-core` - Runtime-agnostic core
3. `bridge-nodejs`, `bridge-bun`, `bridge-deno` - Runtime adapters
4. `config-bridge` - Config loading bridge (outputs to `projects/cli/assets/js/dist/`)

## Troubleshooting

If build fails:

1. Ensure `pnpm install` has been run at project root
2. Check that Nix environment is loaded (`direnv allow`)
3. Try force rebuild: `./xtask.sh build-js --force`

## Output Locations

- Bridge packages: `projects/cli-js-bridges/*/dist/`
- CLI assets: `projects/cli/assets/js/dist/config-bridge.js`
- Release assets: `out/cli/assets/js/dist/`
  </command-instruction>

<current-context>
<bridge_status>
!`ls -la projects/cli-js-bridges/*/dist/*.js 2>/dev/null | head -20 || echo "No built bridges found"`
</bridge_status>
<cli_assets>
!`ls -la projects/cli/assets/js/dist/ 2>/dev/null || echo "No CLI assets found"`
</cli_assets>
</current-context>
