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
4. Appending findings to appropriate `.sisyphus/notepads/` files (plans are orchestrator-managed)

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
   - Config files (Cargo.toml, package.json, etc.) → Dependency changes

3. **Determine documentation updates**:

   | Change Type          | Target Documentation                         |
   | -------------------- | -------------------------------------------- |
   | Bug fixes            | `.sisyphus/notepads/{category}/learnings.md` |
   | New features         | `.sisyphus/notepads/{category}/learnings.md` |
   | Refactoring          | `.sisyphus/notepads/{category}/learnings.md` |
   | Architecture changes | Root `AGENTS.md` + module AGENTS.md          |
   | Build system         | `.sisyphus/notepads/{category}/learnings.md` |
   | Lint fixes           | `.sisyphus/notepads/{category}/learnings.md` |

4. **Generate summary** - Create a structured summary following the format in COMPLETED_TASKS.md:

   ```markdown
   #### ~~任务: [Task Title]~~ ✅ 已完成 (YYYY-MM-DD)

   **文件**: [list of modified files]

   **实现内容**:

   - ✅ [Change 1]
   - ✅ [Change 2]
   ```

5. **Update documentation** - Append to the appropriate section in `.sisyphus/notepads/{category}/learnings.md`

## Documentation Structure

```
.sisyphus/
├── plans/                          # Work plans (git tracked)
│   ├── cli-next-steps.md           # Current priorities
│   ├── completed-stability-quality.md  # Historical archive
│   └── future-enhancements.md      # Backlog
└── notepads/                       # Learning notes (gitignored)
    ├── {category}/
    │   ├── learnings.md            # Accumulated learnings
    │   ├── issues.md               # Known issues and blockers
    │   └── decisions.md            # Architectural decisions
```

## AGENTS.md Hierarchy

```
AGENTS.md (root)           # Project overview, quick commands
├── projects/cli/AGENTS.md        # CLI module details
├── projects/core/AGENTS.md       # Core library details
└── projects/cli-js-bridges/AGENTS.md  # JS bridges details
```

## Guidelines

1. **Be concise** - Summarize changes, don't duplicate code
2. **Use consistent format** - Follow existing documentation patterns
3. **Date entries** - Include completion dates for tracking
4. **Link files** - Reference modified files with relative paths
5. **Categorize properly** - Group related changes together

## Example Output

For a lint fix session, the output might be:

```markdown
### 代码质量清理 (2026-02-XX)

#### ~~任务: Rust Clippy 警告修复~~ ✅ 已完成

**修复内容**: 从 30+ warnings 降至 0 warnings

**文件修改**:

- `projects/cli/src/bridge/manager/mod.rs` - 移除不必要的 Result 包装
- `projects/cli/src/config/loader.rs` - 类型别名重构减少复杂度
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
<docs_last_modified>
!`ls -la .sisyphus/plans/*.md .sisyphus/notepads/*/ 2>/dev/null || echo "No docs found"`
</docs_last_modified>
</current-context>
