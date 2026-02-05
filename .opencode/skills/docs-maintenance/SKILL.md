---
name: docs-maintenance
description: Documentation maintenance and plan archiving - guides AI on when to update docs, how to write consistently, and managing completed plans
---

# Documentation Maintenance

## Quick Reference

- **Update trigger**: Code changes → sync docs via `/sync-docs` command
- **Append-only**: All notepad files use append pattern (`.sisyphus/notepads/`)
- **Never edit**: Plan files (`.sisyphus/plans/*.md`) are read-only
- **Archive completed**: Move finished plans to `.sisyphus/plans/archive/`

## Update Trigger Matrix

| Code Change Location                  | Document to Update                            | Trigger Condition                                |
| ------------------------------------- | --------------------------------------------- | ------------------------------------------------ |
| `projects/cli/src/`                   | `projects/cli/AGENTS.md`                      | Architecture changes, new modules, refactoring   |
| `projects/core/src/`                  | `projects/core/AGENTS.md`                     | API changes, new types, state management updates |
| `projects/cli-js-bridges/`            | `projects/cli-js-bridges/AGENTS.md`           | Service changes, JSON-RPC routing, new bridges   |
| `xtask/src/`                          | Root `AGENTS.md` QUICK COMMANDS section       | Build task changes, new xtask commands           |
| `Cargo.toml` changes                  | Root `AGENTS.md` CONVENTIONS section          | Rust edition, feature flag updates               |
| `package.json` changes                | Root `AGENTS.md` CONVENTIONS section          | Node version, workspace structure changes        |
| Config type exports (`#[derive(TS)]`) | `projects/core/AGENTS.md` + notepad           | Generated TS types, config schema changes        |
| Conventions discovered                | Root `AGENTS.md` + module AGENTS.md           | Project-wide patterns, anti-patterns to document |
| Bug fixes                             | `.sisyphus/notepads/{plan-name}/issues.md`    | Root causes, prevention strategies               |
| Architectural decisions               | `.sisyphus/notepads/{plan-name}/decisions.md` | Rationale, alternatives considered, tradeoffs    |
| Performance discoveries               | `.sisyphus/notepads/{plan-name}/learnings.md` | Bottlenecks found, optimization results          |

## Sync Procedures

### After Completing a Task

1. **Use the command**:

   ```bash
   /sync-docs
   ```

2. **The command will**:
   - Analyze uncommitted changes via `git diff`
   - Identify affected documentation files
   - Guide you through what needs updating

3. **Manual verification**:
   - Review what changed in source files
   - Determine which docs need updates (use trigger matrix above)
   - Append to relevant notepads in structured format

### Sync Checklist

- [ ] All source file changes reviewed
- [ ] Corresponding docs identified (use trigger matrix)
- [ ] `.sisyphus/notepads/{plan-name}/` files appended (never replaced)
- [ ] Entry includes date, changes, verification steps
- [ ] Cross-references use relative paths
- [ ] No code duplication (summarize instead)

### Notepad Append Format

```markdown
## [YYYY-MM-DD] Task Title

### Changes Made

- `path/to/file.rs`: Description of change
- `path/to/file.ts`: Description of change

### Verification

- Command run and result
- Test results

### Key Learnings

- Pattern discovered or gotcha avoided
```

### Documentation Hierarchy

```
AGENTS.md (root)                           # Project overview, quick commands
├── projects/cli/AGENTS.md                 # CLI module quick reference
├── projects/core/AGENTS.md                # Core library quick reference
└── projects/cli-js-bridges/AGENTS.md      # JS bridges quick reference

.sisyphus/
├── plans/                                 # Executable boulder plans (git tracked)
│   ├── {plan-name}.md                     # Active plan
│   └── archive/                           # Completed plans (git tracked)
├── roadmaps/                              # Project planning (git tracked)
│   ├── cli-next-steps.md                  # Priority roadmap
│   ├── p2-metadata-manager-breakdown.md   # Design specs
│   └── future-enhancements.md
└── notepads/                              # Working memory (gitignored)
    └── {plan-name}/
        ├── learnings.md                   # Accumulated learnings
        ├── issues.md                      # Known issues & blockers
        ├── decisions.md                   # Architectural decisions
        └── problems.md                    # Unresolved technical debt
```

## Plan Archiving

Plans move to `.sisyphus/plans/archive/` when:

1. All checkboxes marked complete
2. No remaining open tasks
3. Boulder state cleaned up

### Archiving Checklist

- [ ] All tasks completed (✅ checkboxes)
- [ ] Final `/sync-docs` run to capture completion
- [ ] Notepad finalized with conclusion entry
- [ ] Plan file moved to `archive/` subdirectory
- [ ] `boulder.json` updated if plan was active
- [ ] Changes committed with message: `docs: archive plan {name}`

### Archive Maintenance

Archive location: `.sisyphus/plans/archive/`

**Keep archived plans accessible**:

- Include in git commits (never delete)
- Document completion date in plan frontmatter
- Reference in roadmaps if relevant to future work

**Example archived plan**:

```markdown
# CLI Subcommand Base (Plan 1) [COMPLETED 2026-02-05]

> Archived: Completed baseline CLI subcommand infrastructure
> See `.sisyphus/notepads/cli-subcommand-base/` for implementation details

## TL;DR

...
```

## Document Templates

### AGENTS.md Module Template

Use this when creating module-level AGENTS.md files:

```markdown
# {Module Name} - Agent Documentation

## Overview

Brief description of module's role in NovelSaga (1-2 sentences).

## Key Concepts

- **Concept 1**: Definition and usage
- **Concept 2**: Definition and usage

## Quick Commands

| Task  | Command   |
| ----- | --------- |
| Build | `command` |
| Test  | `command` |

## Architecture

High-level module structure (ASCII diagram or bullet points).

## Conventions

### Pattern Name

- Rule 1
- Rule 2

## Anti-Patterns

| Don't            | Do Instead      |
| ---------------- | --------------- |
| Pattern to avoid | Correct pattern |

## When to Use

Load this documentation when working on:

- File pattern 1
- File pattern 2
```

### Plan Template

Use when creating new `.sisyphus/plans/{name}.md`:

```markdown
# {Plan Title} (Plan N)

## TL;DR

> **Quick Summary**: {One sentence goal}
>
> **Deliverables**:
>
> - Deliverable 1
> - Deliverable 2
>
> **Estimated Effort**: {Small/Medium/Large}
> **Parallel Execution**: {YES/NO}
> **Depends On**: {Plan name or "None"}

---

## Context

### Original Request

{User request or business context}

### Prerequisites

{Any plans or conditions that must be met first}

### Current State

{Current implementation status or code excerpt}

---

## Work Objectives

### Core Objective

{High-level what we're building}

### Concrete Deliverables

- File 1: Description
- File 2: Description

---

## Tasks

- [ ] Task 1
  - [ ] Subtask 1.1
  - [ ] Subtask 1.2
- [ ] Task 2
- [ ] Verification & Documentation

## Notes

{Optional implementation hints, gotchas, or architecture notes}
```

### Roadmap Template

Use for `.sisyphus/roadmaps/{name}.md`:

```markdown
# {Roadmap Title}

## Current Status

- Active plan: {plan name}
- Last updated: {date}
- Completion: {X%}

## Priorities

### P1 (Must Have)

- [ ] Feature 1
- [ ] Feature 2

### P2 (Should Have)

- [ ] Feature 3
- [ ] Feature 4

### P3 (Nice to Have)

- [ ] Feature 5

## Design Notes

{Architecture sketches, API designs, or design decisions}

## Completed

See `.sisyphus/plans/archive/` for finished plans.

---

## Legend

- **P1**: Critical path blocking other work
- **P2**: Important features, deprioritizable
- **P3**: Enhancements and Polish
```

### Notepad Structure Template

Create `.sisyphus/notepads/{plan-name}/` with these files:

**learnings.md** - Accumulated technical insights

```markdown
# {Plan Name} - Learnings

## [YYYY-MM-DD] Initial Setup

- Discovery 1
- Discovery 2

## [YYYY-MM-DD] Task Completion

- Pattern found
- Performance insight
```

**issues.md** - Known problems and blockers

```markdown
# {Plan Name} - Issues & Blockers

## Active Issues

### Issue 1: {Title}

- Status: OPEN / RESOLVED
- Impact: {High/Medium/Low}
- Details: {Description}
- Workaround: {If applicable}

## Resolved

### Issue X: {Title}

- Resolved by: {Solution}
- Date: {YYYY-MM-DD}
```

**decisions.md** - Architectural choices

```markdown
# {Plan Name} - Architecture Decisions

## Decision 1: {Title}

### Context

{What problem did we face?}

### Decision

{What did we choose and why?}

### Alternatives Considered

1. Option A: {pros/cons}
2. Option B: {pros/cons}

### Status

ACCEPTED / DEPRECATED

### Implications

{What does this mean for future work?}
```

## Anti-Patterns

| Don't                                  | Do Instead                                                     | Why                                   |
| -------------------------------------- | -------------------------------------------------------------- | ------------------------------------- |
| Edit `.sisyphus/plans/*.md` directly   | Read plans, use notepad files for notes                        | Plans are orchestrator-managed state  |
| Overwrite notepad files                | Append with `\n\n##` section headers                           | Preserve all learnings and history    |
| Put code snippets in docs              | Link files with relative paths or summarize                    | Prevents documentation drift          |
| Document everything in AGENTS.md       | Use notepads for working memory, AGENTS.md for stable patterns | AGENTS.md should be concise reference |
| Create docs without a plan             | Clarify requirements in plan first                             | Prevents document scope creep         |
| Use absolute paths in cross-references | Use relative paths from repo root                              | Maintainable when project moves       |
| Modify `_config.ts` exports            | Extend in separate files in same location                      | Generated files should not be edited  |
| Document implementation details        | Document design decisions and rationales                       | Details change with refactoring       |

## When to Use

Load this skill when:

- Planning documentation updates after code changes
- Archiving completed work plans
- Setting up documentation for a new plan
- Maintaining consistency across AGENTS.md files
- Reviewing what docs need syncing after a task
- Creating new roadmap or notepad files
- Designing multi-file documentation changes

This skill guides **when and how** to maintain docs. Use `/sync-docs` command to **execute** documentation updates.
