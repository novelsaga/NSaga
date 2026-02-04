# Migration .opencode/docs/ to .sisyphus/ OpenCode Standard

## TL;DR

Migrate existing AI prompt documentation from `.opencode/docs/` (3 files: NEXT_STEPS.md, COMPLETED_TASKS.md, BACKLOG.md) to the official OpenCode `.sisyphus/` directory structure with proper plan files and notepads for knowledge accumulation.

## Context

### Background

The NovelSaga project currently stores work plans and documentation in `.opencode/docs/` (root-level agent storage). OpenCode has an official `.sisyphus/` directory structure that provides standardized state management for orchestration, learning accumulation, and runtime tracking.

### Current State

- **Source location**: `/home/zerozawa/code/src/nsaga/.opencode/docs/`
- **Files to migrate**:
  - `NEXT_STEPS.md` (81 lines) - Active P1/P2/P3 TODOs with Chinese text
  - `COMPLETED_TASKS.md` (276 lines) - Historical task completion records with timestamps
  - `BACKLOG.md` (46 lines) - Deferred items (robustness, performance, infrastructure)
- **Content analysis**:
  - Stability & quality: Clippy fixes, ESLint fixes, E2E tests (30+ → 0 warnings, 42 → 0 errors, 14/14 passing)
  - JS Bridges & build: xtask build-js, type generation path migration, incremental builds
  - CLI Runtime & Config: Runtime discovery (--runtime, --node-path), ConfigLoader refactoring, static format support
  - Backlog: RPC timeout, crash logging, config hot reload, warmup, CI/CD

### Constraints

- **Preserve content**: All original text, timestamps, file references must be maintained
- **No information loss**: Historical archive (COMPLETED_TASKS.md) becomes reference-only plan
- **Git-only plans**: Plans tracked in git; notepads and runtime state ignored
- **Append-only notepads**: Always append findings, never overwrite
- **Original files retained**: `.opencode/docs/` kept as backup, not deleted
- **No breaking changes**: No existing code or tooling modifications required

## Objectives

### Primary Goal

Successfully restructure AI documentation from `.opencode/docs/` to `.sisyphus/` directory structure while preserving all content and following OpenCode conventions.

### Success Criteria

- [ ] All 3 plan files created with proper OpenCode format (Context, Objectives, TODOs, etc.)
- [ ] All 3 notepad directories initialized (12 total notepad files - 4 files × 3 directories)
- [ ] `.gitignore` updated with `.sisyphus/boulder.json` and `.sisyphus/notepads/`
- [ ] All content preserved (timestamps, file paths, Chinese text)
- [ ] No information loss during migration
- [ ] Can execute `/start-work` on created plans
- [ ] Directory structure matches proposal exactly

## Verification Strategy

### Test Approach

Manual verification of each file and directory to ensure:

1. All files created at correct paths
2. Content matches source (with correct transformations)
3. Structure follows OpenCode conventions
4. .gitignore properly excludes runtime state

### Manual Verification Steps

**Step 1: Directory Structure Check**

```bash
ls -la .sisyphus/
ls -la .sisyphus/plans/
ls -la .sisyphus/notepads/stability-and-quality/
ls -la .sisyphus/notepads/cli-runtime-config/
ls -la .sisyphus/notepads/build-automation/
```

Expected: All directories exist with correct structure.

**Step 2: File Content Verification**

```bash
wc -l .sisyphus/plans/*.md
wc -l .sisyphus/notepads/*/*.md
git check-ignore .sisyphus/boulder.json
git check-ignore .sisyphus/notepads/
```

Expected:

- 4 plan files with line counts matching transformed content (including this migration plan)
- 12 notepad files initialized (each with content)
- `.sisyphus/boulder.json` ignored by git
- `.sisyphus/notepads/` ignored by git

**Step 3: Content Preservation Check**

```bash
# Verify source files still exist (as backup)
ls -la .opencode/docs/

# Spot-check content matches
head -20 .sisyphus/plans/cli-next-steps.md
head -20 .sisyphus/notepads/stability-and-quality/learnings.md
```

Expected: All source files intact, migrated content matches original.

**Step 4: Plan Format Validation**

- [ ] All 4 plans have Context section with Background/Current State/Constraints
- [ ] Each plan has Objectives (Primary Goal + Success Criteria checkboxes)
- [ ] TODOs use `- [ ]` format (never `- [x]`)
- [ ] Naming follows kebab-case (e.g., `cli-next-steps.md`)

## TODO List

### Phase 1: Directory Structure (Quick)

- [ ] **Create main .sisyphus directory**
  - Task: `mkdir -p .sisyphus`
  - Category: quick
  - Skills: []
  - Parallelizable: false

- [ ] **Create plans subdirectory**
  - Task: `mkdir -p .sisyphus/plans`
  - Category: quick
  - Skills: []
  - Parallelizable: false

- [ ] **Create notepads subdirectory**
  - Task: `mkdir -p .sisyphus/notepads`
  - Category: quick
  - Skills: []
  - Parallelizable: false

- [ ] **Create 3 notepad category directories**
  - Task: `mkdir -p .sisyphus/notepads/stability-and-quality .sisyphus/notepads/cli-runtime-config .sisyphus/notepads/build-automation`
  - Category: quick
  - Skills: []
  - Parallelizable: false

### Phase 2: Plan Files (Standard)

- [ ] **Create cli-next-steps.md plan**
  - Task: Write `.sisyphus/plans/cli-next-steps.md` from NEXT_STEPS.md
  - Category: writing
  - Skills: []
  - Parallelizable: false
  - Files: `.sisyphus/plans/cli-next-steps.md`

- [ ] **Create completed-stability-quality.md plan**
  - Task: Write `.sisyphus/plans/completed-stability-quality.md` from COMPLETED_TASKS.md
  - Category: writing
  - Skills: []
  - Parallelizable: false
  - Files: `.sisyphus/plans/completed-stability-quality.md`

- [ ] **Create future-enhancements.md plan**
  - Task: Write `.sisyphus/plans/future-enhancements.md` from BACKLOG.md
  - Category: writing
  - Skills: []
  - Parallelizable: false
  - Files: `.sisyphus/plans/future-enhancements.md`

### Phase 3: Notepad Files (Standard)

- [ ] **Initialize stability-and-quality/ notepad directory**
  - Task: Create 4 files in `.sisyphus/notepads/stability-and-quality/`
  - Category: quick
  - Skills: []
  - Parallelizable: false
  - Files: `learnings.md`, `decisions.md`, `issues.md`, `problems.md`

- [ ] **Initialize cli-runtime-config/ notepad directory**
  - Task: Create 4 files in `.sisyphus/notepads/cli-runtime-config/`
  - Category: quick
  - Skills: []
  - Parallelizable: false
  - Files: `learnings.md`, `decisions.md`, `issues.md`, `problems.md`

- [ ] **Initialize build-automation/ notepad directory**
  - Task: Create 4 files in `.sisyphus/notepads/build-automation/`
  - Category: quick
  - Skills: []
  - Parallelizable: false
  - Files: `learnings.md`, `decisions.md`, `issues.md`, `problems.md`

### Phase 4: Git Configuration (Quick)

- [ ] **Update .gitignore for .sisyphus runtime state**
  - Task: Append to `.gitignore` runtime files
  - Category: quick
  - Skills: []
  - Parallelizable: false
  - Files: `.gitignore`

## Dependencies

### File Dependencies

- `.opencode/docs/NEXT_STEPS.md` → source for `cli-next-steps.md`
- `.opencode/docs/COMPLETED_TASKS.md` → source for `completed-stability-quality.md`
- `.opencode/docs/BACKLOG.md` → source for `future-enhancements.md`
- `.gitignore` → target for update
- `.opencode/config.json` → read-only reference for LSP config

### Tool Dependencies

- Basic Bash tools (`mkdir`, `cat`, `echo`)
- Git for `.gitignore` updates
- Text editor for creating plan files

### Required Permissions

- Write access to `.sisyphus/` directory
- Write access to `.gitignore`
- Read-only access to `.opencode/docs/` (source files)

## Risks & Mitigation

| Risk                                                        | Impact | Mitigation                                                        |
| ----------------------------------------------------------- | ------ | ----------------------------------------------------------------- |
| Content transformation errors (timestamp loss, path errors) | Medium | Manual verification using `head`, `grep` after each file creation |
| Incorrect .gitignore rules (ignoring plans)                 | Medium | Test with `git check-ignore` after update                         |
| Notepad files overwritten instead of appended               | High   | Follow strict "append-only" rule, verify after creation           |
| Directory structure mismatch (missing subdirectories)       | Medium | Use absolute paths and verify with `ls -la` after each phase      |
| Source files accidentally deleted                           | High   | NEVER use `rm` on source files; only read                         |

## References

- **Source Files**:
  - `.opencode/docs/NEXT_STEPS.md` (81 lines)
  - `.opencode/docs/COMPLETED_TASKS.md` (276 lines)
  - `.opencode/docs/BACKLOG.md` (46 lines)

- **OpenCode Documentation**:
  - [Oh-My-OpenCode .sisyphus/ structure](https://github.com/code-yeongyu/oh-my-opencode)
  - [Plan file format requirements](https://github.com/code-yeongyu/oh-my-opencode/blob/dev/src/agents/prometheus/plan-template.ts)
  - [Notepad conventions](https://github.com/code-yeongyu/oh-my-opencode/blob/dev/src/agents/atlas/default.ts)

- **Project Documentation**:
  - [AGENTS.md](../../AGENTS.md) - Workspace knowledge base
  - [COMPLETED_TASKS.md](../../.opencode/docs/COMPLETED_TASKS.md) - Historical records
  - [NEXT_STEPS.md](../../.opencode/docs/NEXT_STEPS.md) - Active TODOs
  - [BACKLOG.md](../../.opencode/docs/BACKLOG.md) - Deferred items

## Execution Commands

```bash
# Start work on this plan
/start-work .sisyphus/plans/docs-migration.md

# Or for automated execution
/ralph-loop
```
