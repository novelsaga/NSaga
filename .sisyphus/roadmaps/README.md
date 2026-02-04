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

| File                               | Type             | Status                                     |
| ---------------------------------- | ---------------- | ------------------------------------------ |
| `cli-next-steps.md`                | Priority Roadmap | Active - P1/P2/P3 priorities               |
| `p2-metadata-manager-breakdown.md` | Design Spec      | May be stale - 32 tasks, verify before use |
| `p3-lsp-integration-breakdown.md`  | Design Spec      | May be stale - 28 tasks, depends on P2     |
| `future-enhancements.md`           | Backlog          | Active - long-term wishlist                |

## Usage Notes

1. **Before executing tasks from design breakdowns**: Verify the code design is still current
2. **To convert to executable plan**: Create a new file in `.sisyphus/plans/` with atomic tasks and verification criteria
3. **Roadmaps inform priorities**: Use these to decide what to build next, not how to build it
