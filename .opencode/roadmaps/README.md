# Roadmaps & Design Specs

This directory contains **project planning documents** - priority roadmaps, design breakdowns, and feature wishlists.

## What Belongs Here

| Type                  | Description                                          | Example                            |
| --------------------- | ---------------------------------------------------- | ---------------------------------- |
| **Priority Roadmaps** | P1/P2/P3 prioritized feature lists                   | `cli-next-steps.md`                |
| **Design Breakdowns** | Detailed design specs with task lists (may be stale) | `p2-metadata-manager-breakdown.md` |
| **Backlogs**          | Future enhancement wishlists                         | `future-enhancements.md`           |

## What Does NOT Belong Here

These go in `.sisyphus/plans/`:

- Executable Sisyphus boulder plans
- Atomic task lists ready for `/start-work`
- Plans with `## VERIFICATION CRITERIA` sections

## Current Contents

| File                                  | Type             | Status                                     |
| ------------------------------------- | ---------------- | ------------------------------------------ |
| `cli-next-steps.md`                   | Priority Roadmap | Active - P1/P2/P3 priorities               |
| `p2-metadata-manager-breakdown.md`    | Design Spec      | May be stale - 32 tasks, verify before use |
| `p3-lsp-integration-breakdown.md`     | Design Spec      | May be stale - 28 tasks, depends on P2     |
| `future-enhancements.md`              | Backlog          | Active - long-term wishlist                |
| `metadata-storage-risk-register.md`   | Risk Register    | Resolved - 所有风险已通过统一存储契约修复  |
| `project-status-report-2026-03-19.md` | Status Report    | **最新** - 代码验证后的实际项目状态评估    |

## Usage Notes

1. **Before executing tasks from design breakdowns**: Verify the code design is still current
2. **To convert to executable plan**: Create a new file in `.sisyphus/plans/` with atomic tasks and verification criteria
3. **Roadmaps inform priorities**: Use these to decide what to build next, not how to build it
