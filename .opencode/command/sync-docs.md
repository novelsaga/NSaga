---
description: Scan project changes and sync to AI documentation
argument-hint: [--since <commit|HEAD~N>] [--dry-run]
---

<command-instruction>
Scan recent project changes and synchronize relevant information to AI documentation files.

## Purpose

This command helps maintain up-to-date AI documentation by:

1. Analyzing git changes (staged, unstaged, or since a specific commit)
2. Identifying which documentation files need updates
3. Generating summaries of completed work
4. Appending findings to appropriate `.sisyphus/notepads/{plan-name}/` files

## Usage Scenarios

### After completing a task

```bash
/sync-docs
```

Analyzes all uncommitted changes and updates documentation.

### After a series of commits

```bash
/sync-docs --since HEAD~3
```

Analyzes the last 3 commits and updates documentation.

### Preview without changes

```bash
/sync-docs --dry-run
```

Shows what would be updated without making changes.

## Steps to Execute

1. **Gather changes** - Run git commands to identify modifications:

   ```bash
   # Uncommitted changes
   git diff --stat
   git diff --cached --stat

   # Or since a specific commit
   git diff --stat <commit>..HEAD
   git log --oneline <commit>..HEAD
   ```

2. **Analyze change categories**:
   - `projects/cli/src/` → CLI changes, may need `projects/cli/AGENTS.md` update
   - `projects/cli-js-bridges/` → Bridge changes, may need `projects/cli-js-bridges/AGENTS.md` update
   - `projects/core/src/` → Core changes, may need `projects/core/AGENTS.md` update
   - `xtask/` → Build system changes
   - `.opencode/` → Command/skill changes
   - `.sisyphus/plans/` → Boulder plan changes
   - `.sisyphus/roadmaps/` → Priority/design doc changes
   - Config files (Cargo.toml, package.json, etc.) → Dependency changes

3. **Determine documentation updates**:

   | Change Type          | Target Documentation                          |
   | -------------------- | --------------------------------------------- |
   | Bug fixes            | `.sisyphus/notepads/{plan-name}/learnings.md` |
   | New features         | `.sisyphus/notepads/{plan-name}/learnings.md` |
   | Refactoring          | `.sisyphus/notepads/{plan-name}/learnings.md` |
   | Architecture changes | Root `AGENTS.md` + module AGENTS.md           |
   | Build system         | `.sisyphus/notepads/{plan-name}/learnings.md` |
   | Lint fixes           | `.sisyphus/notepads/{plan-name}/learnings.md` |
   | Priority completion  | `.sisyphus/roadmaps/cli-next-steps.md`        |
   | Design decisions     | `.sisyphus/roadmaps/` design spec files       |

4. **Generate summary** - Create a structured summary:

   ```markdown
   ## [YYYY-MM-DD] Task Title

   ### Changes Made

   - File 1: Description of change
   - File 2: Description of change

   ### Verification

   - Command run and result

   ### Key Learnings

   - Pattern discovered or gotcha avoided
   ```

5. **Update documentation** - Append to `.sisyphus/notepads/{plan-name}/learnings.md`

## Documentation Structure

```
.sisyphus/
├── plans/                              # Executable boulder plans (git tracked)
│   ├── archive/                        # Completed plans
│   │   ├── cli-subcommand-base.md
│   │   ├── completed-stability-quality.md
│   │   └── docs-migration.md
│   ├── cli-init-subcommand.md          # Example: init command plan
│   ├── cli-lsp-network-params.md       # Example: LSP params plan
│   └── ...
├── roadmaps/                           # Project planning docs (git tracked)
│   ├── cli-next-steps.md               # P1/P2/P3 priority roadmap
│   ├── p2-metadata-manager-breakdown.md # Design spec
│   ├── p3-lsp-integration-breakdown.md  # Design spec
│   └── future-enhancements.md          # Backlog/wishlist
├── notepads/                           # Learning notes (gitignored)
│   └── {plan-name}/                    # One folder per active plan
│       ├── learnings.md                # Accumulated learnings
│       ├── issues.md                   # Known issues and blockers
│       └── decisions.md                # Architectural decisions
└── boulder.json                        # Active plan state (gitignored)
```

## AGENTS.md Hierarchy

```
AGENTS.md (root)                       # Project overview, quick commands
├── projects/cli/AGENTS.md             # CLI module details
├── projects/core/AGENTS.md            # Core library details
└── projects/cli-js-bridges/AGENTS.md  # JS bridges details
```

## Guidelines

1. **Be concise** - Summarize changes, don't duplicate code
2. **Use consistent format** - Follow existing documentation patterns
3. **Date entries** - Include completion dates for tracking
4. **Link files** - Reference modified files with relative paths
5. **Match plan name** - Use the active plan name for notepad folder

## Example Output

For a CLI refactor session with plan `cli-subcommand-base`:

```markdown
## [2026-02-05] Task 2 - Update main.rs

### Changes Made

- `projects/cli/src/main.rs`: Added command dispatch match expression
- `projects/cli/src/main.rs`: Added `print_status_info()` function

### Verification

- `cargo build -p novelsaga-cli`: SUCCESS
- `cargo test -p novelsaga-cli`: 15 passed, 0 failed

### Key Learnings

- Match on `Option<Commands>` allows clean None handling for status display
- `todo!()` macros with context show expected args during development
```

</command-instruction>

<current-context>
<git_status>
!`git status --short 2>/dev/null | head -20 || echo "Not a git repository"`
</git_status>
<recent_commits>
!`git log --oneline -5 2>/dev/null || echo "No git history"`
</recent_commits>
<changed_files_since_last_commit>
!`git diff HEAD~1 --stat 2>/dev/null | tail -10 || echo "Cannot determine changes"`
</changed_files_since_last_commit>
<active_boulder>
!`cat .sisyphus/boulder.json 2>/dev/null || echo "No active boulder"`
</active_boulder>
<docs_last_modified>
!`ls -la .sisyphus/plans/*.md .sisyphus/roadmaps/*.md 2>/dev/null | head -15 || echo "No plans found"`
</docs_last_modified>
</current-context>
