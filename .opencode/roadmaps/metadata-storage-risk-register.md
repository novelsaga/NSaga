# Metadata Storage Risk Register

## Context

The current implementation contains mixed path strategies for metadata index storage.
Roadmap/design documents describe a unified project-local cache layout (`.cache/novelsaga/sled`) with metadata documents in `metadata/` scopes, but runtime code currently uses a mix of `.novelsaga/cache/index` and global `ProjectDirs(...)/metadata` paths.

This register tracks concrete hidden risks and why the migration appears partially completed.

## Risk Table

| ID  | Risk                                                                                                                | Current Evidence (Code)                                                                                                                                                                                                                                                            | Planned/Expected (Docs)                                                                                                       | Impact                                                                                          | Likelihood | Suggested Remediation                                                                                                           |
| --- | ------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------- |
| R1  | Index path policy is inconsistent across modules                                                                    | `projects/cli/src/lsp/backend.rs` uses `workspace/.novelsaga/cache/index` (with workspace root) and `ProjectDirs.cache_dir()/metadata` fallback; `projects/cli/src/commands/metadata.rs` uses `.novelsaga/cache/index` for `index` but `ProjectDirs(...)/metadata` for `list/show` | `p2-metadata-manager-breakdown.md` describes `.cache/novelsaga/sled` under workspace and metadata files in `metadata/` scopes | CLI/LSP may read/write different DBs, creating "indexed but cannot list/show" confusion         | High       | Define one canonical resolver in shared module, migrate all callsites (LSP + CLI index/list/show) to it                         |
| R2  | Roadmap target and implementation diverged (`.cache/novelsaga` vs `.novelsaga/cache/index`)                         | Runtime paths in CLI/LSP still point to `.novelsaga/cache/index` and/or global `.../metadata`                                                                                                                                                                                      | `p2-metadata-manager-breakdown.md` explicitly states index in `.cache/novelsaga/` (with `sled/`)                              | Technical debt and operator confusion; documentation no longer trustworthy for storage behavior | High       | Either (A) migrate code to documented target, or (B) update roadmap to match accepted implementation decision                   |
| R3  | Completion claims likely overstated vs current path behavior                                                        | Archived plan `metadata-completion-plan.md` final checklist marks CLI/LSP routing and no sled resource-busy issues as complete, but current code still has multi-path storage and lock-sensitive behavior in E2E history                                                           | Completion checklist implies stable, unified behavior                                                                         | False sense of completion; future work built on incorrect assumptions                           | Medium     | Add a post-completion audit checklist requiring path-consistency proof and runtime invariants before archive                    |
| R4  | Global fallback (`ProjectDirs(...)/metadata`) can hide project index silently                                       | LSP/CLI fallback writes to global metadata DB when project root assumptions fail                                                                                                                                                                                                   | Design intent emphasizes project-scoped metadata hierarchy                                                                    | Debugging complexity and accidental cross-project contamination                                 | Medium     | Require explicit mode switch for global cache, default to project-only with clear warning/error                                 |
| R5  | Namespace/type model is coupled to `metadata/` directory semantics while storage path is decoupled and inconsistent | Core parser/type logic depends on `metadata/` path conventions, but storage resolver differs per entrypoint                                                                                                                                                                        | Roadmap expects coherent metadata scope + storage model                                                                       | Hard-to-reason behavior when indexing from different command paths                              | Medium     | Introduce integration tests that assert same entity visibility across LSP + `index/list/show` under identical workspace         |
| R6  | Lack of single source of truth for metadata index location                                                          | No shared `resolve_metadata_index_path()` used by all consumers                                                                                                                                                                                                                    | Design docs imply unified architecture (CLI+LSP shared IndexManager path policy)                                              | Future regressions likely when adding commands/features                                         | High       | Create shared resolver in `projects/cli/src/metadata/` and delete duplicated path logic in LSP backend and CLI command handlers |

## Evidence References

- Roadmap/design target:
  - `.sisyphus/roadmaps/p2-metadata-manager-breakdown.md` (storage section: `.cache/novelsaga/`, workspace layout sample)
- Archive completion claims:
  - `.sisyphus/plans/archive/metadata-completion-plan.md` (final checklist marked complete)
- Current code path logic:
  - `projects/cli/src/lsp/backend.rs` (`determine_index_path`)
  - `projects/cli/src/commands/metadata.rs` (`determine_db_path`, `handle_list`, `handle_show`)
  - `projects/core/src/config/workspace.rs` (default `.novelsaga/cache` config)

## Suggested Next Decision

Pick one storage contract and enforce it end-to-end:

1. **Project-local only**: `<workspace>/.cache/novelsaga/sled` (or finalized variant), no implicit global fallback.
2. **Dual-mode explicit**: project-local default + opt-in global mode, both clearly surfaced in logs/config.

Until this is decided and implemented, metadata storage should be considered **partially migrated**.


## Remediation Status (RESOLVED)

All identified risks have been addressed through the unified storage contract remediation:

- **R1, R2, R6 (Path Consistency)**: Resolved by implementing a shared `MetadataResolver` in `projects/cli/src/metadata/resolver.rs`. All modules (LSP backend, CLI index/list/show) now use the canonical `<workspace>/.cache/novelsaga/sled` path.
- **R3 (Overstated Completion)**: Resolved by performing a cross-module audit and adding formal path-consistency verification to the integration test suite.
- **R4 (Global Fallback)**: Resolved by removing implicit OS-global fallbacks. Storage is now strictly project-scoped to prevent cross-project contamination and silent data loss.
- **R5 (Model Coupling)**: Resolved by unifying the entrypoint path logic, ensuring that entities indexed via CLI are identical to those visible in the LSP under the same workspace.
- **Legacy Support**: Automatic migration from the old `.novelsaga/cache/index` location is implemented, ensuring no data loss for existing projects while transitioning to the new standard.