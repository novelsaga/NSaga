---
description: Run all linters (Rust clippy + ESLint + Nix formatter check)
argument-hint: [--fix]
---

<command-instruction>
Run comprehensive linting across the entire NovelSaga monorepo.

## Steps

1. **Rust Linting** (Clippy):

   ```bash
   cargo clippy --all-targets -- -D warnings
   ```

2. **TypeScript/JavaScript Linting** (ESLint):

   ```bash
   pnpm exec eslint . --ext .ts,.tsx,.js,.mjs
   ```

   With auto-fix: `pnpm exec eslint . --fix`

3. **Nix Formatting Check** (Alejandra):
   ```bash
   alejandra --check .
   ```

## Fix Mode

When `--fix` argument is provided:

- ESLint auto-fixes style issues
- Run `alejandra .` to format Nix files
- Clippy provides suggestions (apply manually with `cargo clippy --fix`)

## Troubleshooting

- **Clippy errors**: Run `cargo clippy --all-targets` to see full output
- **ESLint not found**: Run `pnpm install` at project root
- **Alejandra not found**: Ensure Nix environment is loaded (`direnv allow`)
  </command-instruction>

<current-context>
<rust_lint_preview>
!`cargo clippy --all-targets 2>&1 | head -20 || echo "Clippy not available"`
</rust_lint_preview>
<eslint_status>
!`pnpm exec eslint --version 2>/dev/null || echo "ESLint not available"`
</eslint_status>
</current-context>
