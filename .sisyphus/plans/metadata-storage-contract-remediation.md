# Metadata Storage Contract Remediation

## TL;DR

> **Summary**: Unify NovelSaga metadata storage onto one project-local contract: source metadata remains in `**/metadata/**/*.md`, while the embedded index lives at `<workspace>/.cache/novelsaga/sled`. Remove divergent CLI/LSP/global path logic, add deterministic legacy migration from workspace-local `.novelsaga/cache/index`, and synchronize the affected roadmap/AGENTS/OpenCode docs.
> **Deliverables**:
>
> - Shared metadata storage resolver and legacy migration helper in `projects/cli/src/metadata/`
> - CLI `index/list/show` and LSP backend using the same canonical path policy
> - TDD coverage proving same-workspace visibility and no global fallback
> - Updated `WorkspaceConfig` defaults and documentation surfaces (`.sisyphus/roadmaps`, `**/AGENTS.md`, `.opencode/command`, `.opencode/skills`)
> - Stage-gated commit/oracle-review workflow for each major step
>   **Effort**: Large
>   **Parallel**: YES - 3 waves + final verification
>   **Critical Path**: Major Step 1 contract/resolver → Major Step 2 runtime adoption → Major Step 3 docs + proof

## Context

### Original Request

- Plan remediation from `.sisyphus/roadmaps/metadata-storage-risk-register.md`.
- Collaborate with Metis during planning.
- Submit the plan to Momus; if review fails, revise and resubmit, or create a new plan file when issues are too numerous.
- Structure work into major steps and detailed substeps.
- Every major step must end with a staged git commit and Oracle review; if Oracle rejects structurally, reset and rewrite that major step instead of patching it.
- Before commits, ensure clippy and ESLint have no warnings/errors and will satisfy git hooks.
- Small steps must be narrowly scoped (≤3 functions/structs) and specify which subagent/task should execute them because executors lack direct edit/bash permissions.

### Interview Summary

- Canonical source-of-truth for metadata files remains `**/metadata/**/*.md`.
- Canonical cache root is project-local: `<workspace>/.cache/novelsaga/`.
- Canonical embedded DB directory name is `sled/` to preserve backend-specific naming.
- No global OS-cache fallback: when no explicit workspace root is available, synthesize the workspace root from execution context (CLI target path, CLI cwd, show target parent, or LSP startup directory).
- Keep the lightweight `metadata/` directory heuristic as a valid workspace-root marker.
- Scope explicitly includes runtime code plus roadmap docs, `**/AGENTS.md`, `.opencode/command`, and `.opencode/skills`.
- Test strategy is TDD.

### Metis Review (gaps addressed)

- Resolved the three blocking choices Metis identified: DB subdirectory (`sled`), no-workspace behavior (context-derived workspace root), and workspace-root heuristic (`metadata/` stays valid).
- Incorporated Metis guardrails: one shared resolver boundary in `projects/cli/src/metadata/`, zero remaining inline `ProjectDirs`/ad-hoc joins in CLI/LSP metadata flows, and no abstraction creep (no traits/builders/backend enum layer in this remediation).
- Expanded scope to cover a hidden correctness gap beyond the risk register: LSP `execute_command` metadata routes currently return placeholder payloads and must be aligned with the same canonical storage policy.

## Work Objectives

### Core Objective

Make every metadata entrypoint (`index`, `list`, `show`, LSP startup/open/update, and LSP execute-command routes) resolve and use the same project-local metadata index contract, while preserving deterministic migration from legacy workspace-local `.novelsaga/cache/index` stores.

### Deliverables

- Shared resolver module under `projects/cli/src/metadata/` that:
  - resolves workspace roots from explicit context or fallback context,
  - returns canonical `<workspace>/.cache/novelsaga/sled`,
  - migrates legacy workspace-local `.novelsaga/cache/index` to the canonical location once per workspace when safe,
  - never falls back to `ProjectDirs(...)/metadata` for metadata indexing.
- CLI metadata commands migrated to the shared resolver.
- LSP backend migrated to the shared resolver, including execute-command metadata handlers.
- `projects/core/src/config/workspace.rs` default cache contract updated to `.cache/novelsaga`.
- Regression tests proving:
  - same workspace → same DB path across CLI/LSP,
  - context-derived fallback works without global cache,
  - legacy workspace-local store migration is deterministic,
  - no remaining direct path constructors remain in metadata CLI/LSP code.
- Documentation sync across roadmap/archive/AGENTS/OpenCode command+skills surfaces.

### Definition of Done (verifiable conditions with commands)

- `cargo test -p novelsaga-cli`
- `cargo test -p novelsaga-core`
- `cargo clippy -p novelsaga-cli --all-targets -- -D warnings`
- `cargo clippy -p novelsaga-core --all-targets -- -D warnings`
- `pnpm exec eslint .`
- `rg -n 'ProjectDirs::from\(|\.novelsaga/cache/index|cache_dir\(\)\.join\("metadata"\)' projects/cli/src/commands/metadata.rs projects/cli/src/lsp/backend.rs` returns no matches related to active metadata path construction
- `rg -n 'implementation pending|todo!\(|Handlers are currently `todo!\(\)` stubs' projects/cli/src/lsp/backend.rs projects/cli/AGENTS.md` returns no metadata-command stale status matches

### Must Have

- One concrete resolver implementation in `projects/cli/src/metadata/`; no duplicate path policy logic.
- Canonical workspace cache path `<workspace>/.cache/novelsaga/sled`.
- Legacy migration from workspace-local `.novelsaga/cache/index` to canonical path.
- Context-derived fallback root when no explicit workspace root exists.
- TDD for resolver, migration, CLI/LSP consistency, and documentation updates.
- Stage commits and Oracle gate after each major step.

### Must NOT Have (guardrails, AI slop patterns, scope boundaries)

- No global `ProjectDirs(...)/metadata` fallback for metadata index reads/writes.
- No new storage abstraction layer (no trait hierarchy, no backend registry, no mode enum beyond narrowly scoped helper data if absolutely required by Rust ergonomics).
- No changes to metadata document schema, namespace/type semantics, or non-metadata bridge/runtime behavior.
- No broad refactors outside `projects/core/src/config/workspace.rs`, `projects/cli/src/commands/metadata.rs`, `projects/cli/src/lsp/backend.rs`, `projects/cli/src/metadata/`, and the explicitly named documentation surfaces.
- No structural Oracle rejection may be “patched”; those major-step branches must be reset and rewritten from the previous accepted commit.

## Verification Strategy

> ZERO HUMAN INTERVENTION — all verification is agent-executed.

- Test decision: TDD + existing Rust same-file tests and xtask/LSP E2E structure
- QA policy: Every task includes exact command-driven happy-path and edge-case validation
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`

## Execution Strategy

### Parallel Execution Waves

> Target: 5-8 tasks per wave. <3 per wave (except final) = under-splitting.
> Extract shared dependencies as Wave-1 tasks for max parallelism.

Wave 1: Major Step 1 — contract, resolver, migration helper, config default
Wave 2: Major Step 2 — CLI/LSP callsite adoption and duplicate-path removal
Wave 3: Major Step 3 — integration proof, documentation sync, final major-step checkpoint

### Dependency Matrix (full, all tasks)

| Task | Depends On | Blocks          |
| ---- | ---------- | --------------- |
| 1    | -          | 2, 3, 4         |
| 2    | 1          | 4, 6, 7, 8      |
| 3    | 1          | 4, 13           |
| 4    | 2, 3       | 5               |
| 5    | 4          | 6, 7, 8, 9, 10  |
| 6    | 5          | 7, 8, 9, 10, 11 |
| 7    | 5          | 9, 10, 11       |
| 8    | 5          | 9, 10, 11       |
| 9    | 6, 7, 8    | 10, 11, 12      |
| 10   | 6, 7, 8, 9 | 11, 12          |
| 11   | 9, 10      | 12              |
| 12   | 11         | 13, 14          |
| 13   | 3, 12      | 14              |
| 14   | 12, 13     | F1-F4           |

### Agent Dispatch Summary (wave → task count → categories)

- Wave 1 → 5 tasks → `quick`, `unspecified-high`
- Wave 2 → 6 tasks → `quick`, `unspecified-high`, `deep`
- Wave 3 → 3 tasks → `unspecified-high`, `writing`, `deep`
- Final Verification → 4 tasks → `oracle`, `unspecified-high`, `deep`

## TODOs

> Implementation + Test = ONE task. Never separate.
> EVERY task MUST have: Agent Profile + Parallelization + QA Scenarios.

- [x] 1. Major Step 1A — Freeze canonical metadata storage contract and executor guardrails

  **What to do**: Add/adjust the execution-facing contract notes and any directly adjacent code comments needed so implementers treat `<workspace>/.cache/novelsaga/sled` as the sole canonical DB path, `**/metadata/**/*.md` as the source tree, and context-derived workspace fallback as the only non-explicit root resolution path. Define the exact “rewrite instead of patch” reset rule that applies after Oracle rejection for the whole major step.
  **Must NOT do**: Do not touch runtime path construction yet. Do not introduce storage abstractions or mode systems. Do not update unrelated docs.

  **Recommended Agent Profile**:
  - Category: `writing` — Reason: contract/guardrail specification pass.
  - Skills: [`docs-maintenance`, `project-architecture`] — align implementation constraints and authoritative wording.
  - Omitted: [`cli-dev`] — no runtime edits in this task.

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: [2, 3, 4] | Blocked By: []

  **References** (executor has NO interview context — be exhaustive):
  - `.sisyphus/roadmaps/metadata-storage-risk-register.md:14-39` — current risks and suggested remediation.
  - `.sisyphus/roadmaps/p2-metadata-manager-breakdown.md:83-87,164-167` — canonical target `.cache/novelsaga/sled`.
  - `.sisyphus/plans/archive/metadata-completion-plan.md:72-78` — stale completion claims that must not drive execution.
  - `projects/core/src/config/workspace.rs:12-20` — current divergent config default.

  **Acceptance Criteria** (agent-executable only):
  - [x] The plan/execution notes explicitly name canonical cache root, DB subdir, fallback rule, and Oracle reset rule.
  - [x] No runtime source file changes are required by this task.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Contract notes capture all mandatory decisions
    Tool: Read
    Steps: Open the plan file and verify it contains canonical path, fallback behavior, and rewrite-on-rejection rule.
    Expected: All decisions are present and unambiguous.
    Evidence: .sisyphus/evidence/task-1-contract-notes.txt

  Scenario: Step remains non-runtime
    Tool: Read
    Steps: Verify this task only targets plan/execution guidance and not runtime files.
    Expected: No runtime file paths appear under this task's output scope.
    Evidence: .sisyphus/evidence/task-1-nonruntime.txt
  ```

  **Commit**: NO | Message: `n/a` | Files: [`.sisyphus/plans/metadata-storage-contract-remediation.md`]

- [x] 2. Major Step 1B — Add resolver tests first in `projects/cli/src/metadata/`

  **What to do**: Create same-file tests for the forthcoming shared resolver covering: explicit workspace root → `<workspace>/.cache/novelsaga/sled`; fallback root from CLI/LSP context when no workspace is passed; `metadata/` heuristic discovery; and legacy workspace-local `.novelsaga/cache/index` detection/migration preconditions. Restrict edits to the new resolver file/module and its colocated `#[cfg(test)]` block.
  **Must NOT do**: Do not migrate CLI/LSP callsites yet. Do not alter `IndexManager` behavior. Do not read/write any global OS cache path.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: one focused module with tests-first scope.
  - Skills: [`testing-guide`, `cli-dev`, `rust-dev`] — same-file Rust tests and CLI-side module placement.
  - Omitted: [`lsp-dev`] — no LSP callsites changed yet.

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: [4] | Blocked By: [1]

  **References** (executor has NO interview context — be exhaustive):
  - `projects/cli/src/metadata/mod.rs:1-12` — metadata module entrypoint.
  - `projects/cli/src/metadata/index.rs:331-599` — same-file test style.
  - `.opencode/skills/testing-guide/SKILL.md:20-53` — inline `#[cfg(test)]` requirement.
  - `projects/cli/src/commands/metadata.rs:183-210` and `projects/cli/src/lsp/backend.rs:116-138` — current duplicate path logic to replace later.

  **Acceptance Criteria** (agent-executable only):
  - [x] New resolver tests exist and fail before implementation logic is added.
  - [x] Tests cover canonical path, context-derived fallback, `metadata/` heuristic, and legacy-path migration preconditions.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Resolver tests are present and fail first
    Tool: Bash
    Steps: Run `cargo test -p novelsaga-cli resolver -- --nocapture` immediately after adding tests and before implementation.
    Expected: At least one new resolver test fails because implementation is not complete yet.
    Evidence: .sisyphus/evidence/task-2-red.log

  Scenario: Tests remain same-file and local to metadata module
    Tool: Read
    Steps: Inspect the new resolver file/module and confirm tests live in `#[cfg(test)] mod tests` in the same file.
    Expected: No separate test file exists.
    Evidence: .sisyphus/evidence/task-2-test-layout.txt
  ```

  **Commit**: NO | Message: `n/a` | Files: [`projects/cli/src/metadata/*`]

- [x] 3. Major Step 1C — Update `WorkspaceConfig` default contract to `.cache/novelsaga`

  **What to do**: Change `projects/core/src/config/workspace.rs` default `cache_dir` from `.novelsaga/cache` to `.cache/novelsaga`, and add or adjust same-file tests if needed to prove the new default contract.
  **Must NOT do**: Do not change unrelated config fields. Do not regenerate TS bindings. Do not update CLI/LSP code here.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: single-struct default change with tight scope.
  - Skills: [`core-dev`, `rust-dev`, `testing-guide`] — config defaults and same-file tests.
  - Omitted: [`cli-dev`] — CLI adoption is later.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: [4, 13] | Blocked By: [1]

  **References** (executor has NO interview context — be exhaustive):
  - `projects/core/src/config/workspace.rs:7-20` — current default.
  - `projects/core/AGENTS.md` WorkspaceConfig documentation.
  - `.sisyphus/roadmaps/p2-metadata-manager-breakdown.md:83-87,164-167` — design target.

  **Acceptance Criteria** (agent-executable only):
  - [x] `WorkspaceConfig::default().cache_dir == ".cache/novelsaga"`.
  - [x] `cargo test -p novelsaga-core` passes.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Core default reflects canonical cache root
    Tool: Bash
    Steps: Run `cargo test -p novelsaga-core workspace -- --nocapture` after adding/asserting the default.
    Expected: Tests confirm `.cache/novelsaga` as the default cache dir.
    Evidence: .sisyphus/evidence/task-3-core-default.log

  Scenario: No unrelated config behavior changes
    Tool: Read
    Steps: Inspect `projects/core/src/config/workspace.rs` scope after editing.
    Expected: Only cache-dir contract and directly related tests changed.
    Evidence: .sisyphus/evidence/task-3-scope.txt
  ```

  **Commit**: NO | Message: `n/a` | Files: [`projects/core/src/config/workspace.rs`]

- [x] 4. Major Step 1D — Implement shared resolver and legacy workspace-local migration helper

  **What to do**: Implement one concrete resolver API in `projects/cli/src/metadata/` that resolves the workspace root from explicit path or contextual fallback, keeps the `metadata/` heuristic, returns `<workspace>/.cache/novelsaga/sled`, and performs deterministic compatibility handling for legacy workspace-local `.novelsaga/cache/index` data. Compatibility rule: if canonical path does not exist and legacy workspace-local path exists, migrate/import the legacy DB into canonical workspace-local `sled` and emit a clear log message; never import from global cache.
  **Must NOT do**: Do not touch CLI/LSP callsites yet. Do not preserve `ProjectDirs(...)/metadata` fallback. Do not create a backend abstraction layer.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: core behavior change with tight boundaries.
  - Skills: [`cli-dev`, `rust-dev`, `testing-guide`] — CLI-side module design, file/path handling, tests-first completion.
  - Omitted: [`lsp-dev`] — LSP adoption is deferred.

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: [5] | Blocked By: [2, 3]

  **References** (executor has NO interview context — be exhaustive):
  - `projects/cli/src/commands/metadata.rs:183-210` — current CLI path logic.
  - `projects/cli/src/lsp/backend.rs:116-138` — current LSP path logic.
  - `projects/cli/src/metadata/mod.rs:1-12` — export location.
  - `.sisyphus/roadmaps/metadata-storage-risk-register.md:14-19` — single-source-of-truth requirement.
  - `.sisyphus/roadmaps/p2-metadata-manager-breakdown.md:164-167` — canonical directory layout.

  **Acceptance Criteria** (agent-executable only):
  - [x] Resolver returns `<workspace>/.cache/novelsaga/sled` for explicit workspace roots.
  - [x] Resolver derives a workspace root from provided CLI/LSP context when no workspace root is supplied.
  - [x] Resolver recognizes `metadata/` as a valid workspace marker.
  - [x] Legacy `.novelsaga/cache/index` under the same workspace is migrated only when canonical path is absent.
  - [x] Resolver tests introduced in Task 2 pass.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Resolver green after implementation
    Tool: Bash
    Steps: Run `cargo test -p novelsaga-cli resolver -- --nocapture` after implementing the resolver.
    Expected: All resolver-related tests pass.
    Evidence: .sisyphus/evidence/task-4-green.log

  Scenario: Legacy path migration is deterministic
    Tool: Bash
    Steps: Run the migration-focused resolver test that creates `.novelsaga/cache/index`, leaves canonical path absent, invokes resolver/migration, then verifies canonical `sled` exists and legacy path no longer remains authoritative.
    Expected: Migration succeeds only in the expected precondition case.
    Evidence: .sisyphus/evidence/task-4-migration.log
  ```

  **Commit**: NO | Message: `n/a` | Files: [`projects/cli/src/metadata/*`, `projects/core/src/config/workspace.rs`]

- [x] 5. Major Step 1E — Major-step checkpoint: lint/test gate, stage commit, Oracle review, reset rule

  **What to do**: After Tasks 1-4 are complete, run the full pre-commit gate for the touched crates, stage only Major Step 1 files, create the major-step commit, then submit the result to Oracle for architecture review. If Oracle reports a structural rejection (duplicate resolver logic, implicit global fallback, unresolved migration semantics, or config/runtime contract mismatch), reset this major step to the last accepted commit and rewrite Tasks 2-4 from scratch.
  **Must NOT do**: Do not continue into CLI/LSP adoption before Oracle approval. Do not patch a structural rejection in place.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: validation + review gate.
  - Skills: [`git-master`, `testing-guide`, `project-architecture`] — commit hygiene and architecture review packaging.
  - Omitted: [`docs-maintenance`] — documentation sync is later.

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: [6, 7, 8] | Blocked By: [4]

  **References** (executor has NO interview context — be exhaustive):
  - `AGENTS.md:82-85` — commit and hook expectations.
  - This plan’s Oracle gate criteria for structural rejection.

  **Acceptance Criteria** (agent-executable only):
  - [x] `cargo test -p novelsaga-cli` passes.
  - [x] `cargo test -p novelsaga-core` passes.
  - [x] `cargo clippy -p novelsaga-cli --all-targets -- -D warnings` passes.
  - [x] `cargo clippy -p novelsaga-core --all-targets -- -D warnings` passes.
  - [x] `pnpm exec eslint .` passes.
  - [x] Major Step 1 commit is created before Oracle review.
  - [x] Oracle approves or the step is reset and rewritten.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Pre-commit gate is clean
    Tool: Bash
    Steps: Run the full test/clippy/eslint gate defined in Commit Strategy.
    Expected: All commands exit 0 with no warnings/errors.
    Evidence: .sisyphus/evidence/task-5-precommit.log

  Scenario: Oracle review decides step fate
    Tool: task/oracle
    Steps: Submit the Major Step 1 diff/summary to Oracle and capture verdict.
    Expected: Oracle returns APPROVE, or the executor resets to the previous accepted commit and restarts Major Step 1.
    Evidence: .sisyphus/evidence/task-5-oracle.txt
  ```

  **Commit**: YES | Message: `refactor(metadata): establish canonical storage contract` | Files: [`projects/cli/src/metadata/*`, `projects/core/src/config/workspace.rs`]

- [x] 6. Major Step 2A — Migrate CLI `handle_index` to the shared resolver

  **What to do**: Replace `determine_db_path()` usage in `handle_index` with the shared resolver, using `IndexCommand.path` as the primary workspace context. Remove direct path construction from the indexing flow and keep all indexing logic otherwise unchanged.
  **Must NOT do**: Do not alter `process_file()`, entity schema, or output format except for updated DB-path display text if necessary. Do not touch `handle_list`, `handle_show`, or LSP in this task.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: single handler migration.
  - Skills: [`cli-dev`, `rust-dev`, `testing-guide`] — command handler wiring and same-file tests.
  - Omitted: [`lsp-dev`] — no LSP changes here.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: [9] | Blocked By: [5]

  **References** (executor has NO interview context — be exhaustive):
  - `projects/cli/src/commands/metadata.rs:107-180` — current `handle_index` implementation.
  - `projects/cli/src/commands/metadata.rs:183-210` — helper being superseded.
  - Shared resolver from Major Step 1.

  **Acceptance Criteria** (agent-executable only):
  - [x] `handle_index` opens the DB via the shared resolver.
  - [x] Indexing a workspace with `metadata/` produces/uses `<workspace>/.cache/novelsaga/sled`.
  - [x] Existing indexing behavior besides path selection remains intact.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: CLI index uses canonical path
    Tool: Bash
    Steps: Run a targeted CLI test that indexes a temp workspace containing `metadata/` and inspect the resulting DB path.
    Expected: DB opens under `<workspace>/.cache/novelsaga/sled`.
    Evidence: .sisyphus/evidence/task-6-index.log

  Scenario: No legacy direct path remains in handle_index
    Tool: Read
    Steps: Inspect the `handle_index` section after migration.
    Expected: It delegates to the shared resolver instead of building `.novelsaga/cache/index` directly.
    Evidence: .sisyphus/evidence/task-6-scope.txt
  ```

  **Commit**: NO | Message: `n/a` | Files: [`projects/cli/src/commands/metadata.rs`]

- [x] 7. Major Step 2B — Migrate CLI `handle_list` to the shared resolver and remove global-cache reads

  **What to do**: Rework `handle_list` so it resolves the same workspace-local DB path as `handle_index`, using process cwd / contextual fallback when no workspace root is passed in arguments. Remove direct `ProjectDirs(...)/metadata` usage entirely.
  **Must NOT do**: Do not change table formatting helpers or CLI output ordering. Do not touch `handle_show` or LSP here.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: one handler, one behavior swap.
  - Skills: [`cli-dev`, `rust-dev`, `testing-guide`] — command resolution and regression testing.
  - Omitted: [`docs-maintenance`] — documentation updates are later.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: [9] | Blocked By: [5]

  **References** (executor has NO interview context — be exhaustive):
  - `projects/cli/src/commands/metadata.rs:249-286` — current `handle_list` global-cache logic.
  - Shared resolver contract from Major Step 1.
  - `.sisyphus/roadmaps/metadata-storage-risk-register.md:14,17,19` — R1/R4/R6.

  **Acceptance Criteria** (agent-executable only):
  - [x] `handle_list` uses the same resolver as `handle_index`.
  - [x] No `ProjectDirs` reference remains in `handle_list`.
  - [x] Running list in a metadata-only workspace sees entities indexed by the same workspace.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: List sees index output from same workspace
    Tool: Bash
    Steps: Index a temp workspace, then run the list-focused test from the same workspace context.
    Expected: Listed entities include the just-indexed metadata from canonical `sled` path.
    Evidence: .sisyphus/evidence/task-7-list.log

  Scenario: Global fallback removed from list
    Tool: Grep
    Steps: Search `projects/cli/src/commands/metadata.rs` for `ProjectDirs::from` after migration.
    Expected: No remaining `ProjectDirs::from` usage tied to list behavior.
    Evidence: .sisyphus/evidence/task-7-grep.txt
  ```

  **Commit**: NO | Message: `n/a` | Files: [`projects/cli/src/commands/metadata.rs`]

- [x] 8. Major Step 2C — Migrate CLI `handle_show` to the shared resolver and keep ID lookup stable

  **What to do**: Rework `handle_show` to resolve the canonical workspace-local DB path based on the target file path context, while preserving canonical-path hashing for entity IDs. Remove direct `ProjectDirs(...)/metadata` usage.
  **Must NOT do**: Do not change entity-ID generation semantics or JSON output schema. Do not touch list/index/LSP in this task.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: one handler migration with stable hashing semantics.
  - Skills: [`cli-dev`, `rust-dev`, `testing-guide`] — targeted handler swap.
  - Omitted: [`lsp-dev`] — no LSP changes.

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: [9] | Blocked By: [5]

  **References** (executor has NO interview context — be exhaustive):
  - `projects/cli/src/commands/metadata.rs:428-465` — current `handle_show` behavior.
  - `projects/cli/src/metadata/index.rs:508-527` — reverse path-index lookup behavior.
  - Shared resolver contract from Major Step 1.

  **Acceptance Criteria** (agent-executable only):
  - [x] `handle_show` uses the shared resolver.
  - [x] Entity ID generation still comes from the canonicalized target file path.
  - [x] A file indexed in a workspace can be shown from the same workspace without global cache access.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Show resolves indexed entity from canonical workspace DB
    Tool: Bash
    Steps: Index a temp workspace, then run a targeted show test on one metadata file.
    Expected: `handle_show` retrieves the entity from `<workspace>/.cache/novelsaga/sled` and prints JSON.
    Evidence: .sisyphus/evidence/task-8-show.log

  Scenario: ID generation remains stable
    Tool: Bash
    Steps: Run the metadata command test that compares generated ID before/after resolver migration for the same canonical path.
    Expected: The entity ID is unchanged.
    Evidence: .sisyphus/evidence/task-8-id.log
  ```

  **Commit**: NO | Message: `n/a` | Files: [`projects/cli/src/commands/metadata.rs`]

- [x] 9. Major Step 2D — Migrate LSP backend startup/index manager wiring to the shared resolver

  **What to do**: Replace `Backend::determine_index_path()` / `open_index_manager()` path construction with the shared resolver and context-derived workspace logic. The LSP startup directory / initialization root becomes the contextual fallback root when editor-provided workspace root is absent.
  **Must NOT do**: Do not change document parsing semantics, watched-file registration logic, or non-metadata LSP capabilities. Do not keep the old path helper as dead code.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: backend path wiring affects runtime initialization.
  - Skills: [`lsp-dev`, `cli-dev`, `rust-dev`, `testing-guide`] — LSP init wiring plus shared CLI module usage.
  - Omitted: [`core-dev`] — no core state changes here.

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: [10, 11] | Blocked By: [6, 7, 8]

  **References** (executor has NO interview context — be exhaustive):
  - `projects/cli/src/lsp/backend.rs:87-138` — current workspace derivation and path helpers.
  - `xtask/src/tasks/test/lsp.rs:150-231` — watched-files LSP E2E pattern.

  **Acceptance Criteria** (agent-executable only):
  - [x] LSP index manager opens the same canonical DB path as CLI for the same workspace.
  - [x] `determine_index_path` no longer exists in `backend.rs`.
  - [x] Existing watched-files metadata behavior still passes against the canonical path.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: LSP uses canonical workspace DB
    Tool: Bash
    Steps: Run the relevant LSP E2E test after adapting expected index directory to `.cache/novelsaga/sled`.
    Expected: Watched-file indexing writes to canonical path and tests pass.
    Evidence: .sisyphus/evidence/task-9-lsp.log

  Scenario: Old backend helper removed
    Tool: Grep
    Steps: Search `projects/cli/src/lsp/backend.rs` for `determine_index_path`.
    Expected: No matches remain.
    Evidence: .sisyphus/evidence/task-9-grep.txt
  ```

  **Commit**: NO | Message: `n/a` | Files: [`projects/cli/src/lsp/backend.rs`, `xtask/src/tasks/test/lsp.rs`]

- [x] 10. Major Step 2E — Replace placeholder LSP execute-command metadata routes with canonical resolver-backed behavior

  **What to do**: Implement the `novelsaga/index`, `novelsaga/list`, and `novelsaga/show` LSP execute-command branches so they stop returning placeholder payloads and instead delegate through the same metadata command/runtime path contract used elsewhere. Ensure any path resolution inside these flows still routes through the shared resolver.
  **Must NOT do**: Do not redesign the LSP command protocol. Do not change command names or JSON-RPC method shape. Do not introduce divergent metadata-only LSP storage logic.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: hidden runtime gap with protocol-facing code.
  - Skills: [`lsp-dev`, `cli-dev`, `rust-dev`, `testing-guide`] — execute-command flow and reuse of command behavior.
  - Omitted: [`docs-maintenance`] — documentation updates come later.

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: [11, 12] | Blocked By: [9]

  **References** (executor has NO interview context — be exhaustive):
  - `projects/cli/src/lsp/backend.rs:677-723` — placeholder execute-command behavior.
  - `projects/cli/src/commands/metadata.rs:55-105` — command routing to `workspace/executeCommand`.
  - Grep results showing command registration points in `backend.rs`.

  **Acceptance Criteria** (agent-executable only):
  - [x] LSP execute-command metadata routes no longer return placeholder “implementation pending” messages.
  - [x] Execute-command metadata routes use the same resolver-backed storage policy as direct CLI/LSP metadata operations.
  - [x] Command round-trips continue to use existing command names and argument shapes.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Execute-command metadata routes perform real work
    Tool: Bash
    Steps: Run or add an LSP-focused test that invokes `workspace/executeCommand` for `novelsaga/index|list|show` against a temp workspace.
    Expected: Returned payloads reflect real indexing/list/show behavior rather than placeholder messages.
    Evidence: .sisyphus/evidence/task-10-exec.log

  Scenario: Placeholder strings removed
    Tool: Grep
    Steps: Search `projects/cli/src/lsp/backend.rs` for `implementation pending`.
    Expected: No metadata execute-command placeholder text remains.
    Evidence: .sisyphus/evidence/task-10-grep.txt
  ```

  **Commit**: NO | Message: `n/a` | Files: [`projects/cli/src/lsp/backend.rs`, `projects/cli/src/commands/metadata.rs`]

- [x] 11. Major Step 2F — Remove duplicate path helpers and prove zero direct metadata path constructors remain

  **What to do**: Delete `determine_db_path()` and any now-dead metadata path helper code after all callsites are migrated. Add/adjust targeted verification so the repository can prove there are no direct metadata path constructors left in CLI/LSP metadata flows.
  **Must NOT do**: Do not delete helper code before all callsites are migrated. Do not widen grep scope to unrelated asset/cache logic.

  **Recommended Agent Profile**:
  - Category: `quick` — Reason: cleanup is mechanical once adoption is complete.
  - Skills: [`cli-dev`, `lsp-dev`, `testing-guide`] — callsite cleanup and verification.
  - Omitted: [`core-dev`] — no core changes.

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: [12] | Blocked By: [9, 10]

  **References** (executor has NO interview context — be exhaustive):
  - `projects/cli/src/commands/metadata.rs:183-210` — helper to remove.
  - `projects/cli/src/lsp/backend.rs:116-138` — helper to remove.
  - Metis directive: prove zero remaining inline `ProjectDirs::from.*join("metadata")` and zero old helper references.

  **Acceptance Criteria** (agent-executable only):
  - [x] `determine_db_path` is removed from `projects/cli/src/commands/metadata.rs`.
  - [x] `determine_index_path` is removed from `projects/cli/src/lsp/backend.rs`.
  - [x] `rg -n 'ProjectDirs::from\(|\.novelsaga/cache/index|cache_dir\(\)\.join\("metadata"\)' projects/cli/src/commands/metadata.rs projects/cli/src/lsp/backend.rs` returns no active metadata path-construction matches.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Duplicate helpers are gone
    Tool: Grep
    Steps: Search for `determine_db_path` and `determine_index_path` in their former files.
    Expected: No matches remain.
    Evidence: .sisyphus/evidence/task-11-helpers.txt

  Scenario: No direct path constructors remain in metadata CLI/LSP flows
    Tool: Bash
    Steps: Run the exact `rg` command from Acceptance Criteria.
    Expected: Command returns no offending matches.
    Evidence: .sisyphus/evidence/task-11-rg.log
  ```

  **Commit**: NO | Message: `n/a` | Files: [`projects/cli/src/commands/metadata.rs`, `projects/cli/src/lsp/backend.rs`]

- [x] 12. Major Step 2G — Major-step checkpoint: runtime gate, stage commit, Oracle review, reset rule

  **What to do**: After Tasks 6-11, run the full test/clippy/eslint gate, stage only Major Step 2 runtime files, commit them, and submit the runtime behavior to Oracle. If Oracle finds structural divergence (duplicate path constructor, surviving global fallback, partial LSP/CLI mismatch, or placeholder command path), reset to the Major Step 1 accepted commit and rewrite Major Step 2 in full.
  **Must NOT do**: Do not begin documentation sync before Oracle approves the runtime step.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: main runtime acceptance gate.
  - Skills: [`git-master`, `testing-guide`, `project-architecture`, `lsp-dev`] — commit, verification, and review packaging.
  - Omitted: [`docs-maintenance`] — docs are next.

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: [13, 14] | Blocked By: [11]

  **References** (executor has NO interview context — be exhaustive):
  - Commit Strategy section in this plan.
  - Oracle structural rejection rules defined in this major step.

  **Acceptance Criteria** (agent-executable only):
  - [x] Full pre-commit gate passes.
  - [x] Major Step 2 runtime commit is created.
  - [x] Oracle approves or the step is reset and rewritten from Major Step 1.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Runtime pre-commit gate is clean
    Tool: Bash
    Steps: Run the full gate from Commit Strategy after Tasks 6-11.
    Expected: All commands exit 0.
    Evidence: .sisyphus/evidence/task-12-precommit.log

  Scenario: Oracle validates end-to-end runtime consistency
    Tool: task/oracle
    Steps: Submit the runtime diff and behavior summary to Oracle.
    Expected: Oracle approves the major step, or the executor resets and rewrites the full major step.
    Evidence: .sisyphus/evidence/task-12-oracle.txt
  ```

  **Commit**: YES | Message: `fix(metadata): migrate cli and lsp to shared resolver` | Files: [`projects/cli/src/commands/metadata.rs`, `projects/cli/src/lsp/backend.rs`, `xtask/src/tasks/test/lsp.rs`, `projects/cli/src/metadata/*`]

- [x] 13. Major Step 3A — Add final integration proof for CLI/LSP path consistency and legacy migration

  **What to do**: Add or extend integration/E2E coverage so one temp workspace proves: legacy `.novelsaga/cache/index` migration into `.cache/novelsaga/sled`, CLI `index/list/show` all see the same entities, and LSP watched-file / execute-command flows use the same DB. Keep this step focused on proof, not behavior redesign.
  **Must NOT do**: Do not alter resolver semantics unless a test exposes a clear bug; if a structural bug appears, stop and route back through the prior major-step rewrite rule instead of opportunistic patching.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: cross-entrypoint integration proof.
  - Skills: [`testing-guide`, `lsp-dev`, `cli-dev`, `project-architecture`] — end-to-end coverage across modules.
  - Omitted: [`docs-maintenance`] — pure verification task.

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: [14] | Blocked By: [3, 12]

  **References** (executor has NO interview context — be exhaustive):
  - `xtask/src/tasks/test/lsp.rs:150-231` — watched-files LSP E2E baseline.
  - `projects/cli/src/commands/metadata.rs:467-498` — current metadata command test baseline.
  - Resolver/migration tests from Major Step 1.

  **Acceptance Criteria** (agent-executable only):
  - [x] A single integration or E2E flow proves CLI and LSP use the same canonical DB in one workspace.
  - [x] Legacy workspace-local migration is covered in that proof or a directly adjacent integration test.
  - [x] No test depends on global cache directories.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Unified workspace proof passes
    Tool: Bash
    Steps: Run the new/extended integration or xtask E2E test covering CLI + LSP consistency in one workspace.
    Expected: CLI and LSP both observe the same entities from canonical `sled` storage.
    Evidence: .sisyphus/evidence/task-13-integration.log

  Scenario: No global cache dependency remains
    Tool: Grep
    Steps: Search the new tests for `ProjectDirs` or home-cache hardcoding.
    Expected: No test fixture depends on global cache paths.
    Evidence: .sisyphus/evidence/task-13-grep.txt
  ```

  **Commit**: NO | Message: `n/a` | Files: [`xtask/src/tasks/test/lsp.rs`, `projects/cli/src/commands/metadata.rs`, `projects/cli/src/metadata/*`]

- [x] 14. Major Step 3B — Synchronize roadmap, archive status, AGENTS, and OpenCode command/skill docs

  **What to do**: Update every documentation surface named in scope so they reflect the real post-remediation contract. Required surfaces: `.sisyphus/roadmaps/metadata-storage-risk-register.md`, `.sisyphus/roadmaps/p2-metadata-manager-breakdown.md`, `.sisyphus/plans/archive/metadata-completion-plan.md` (mark stale claims / add audit note, not fake completion), root `AGENTS.md`, `projects/cli/AGENTS.md`, any other `**/AGENTS.md` whose storage guidance changes, `.opencode/command/sync-docs.md`, `.opencode/skills/docs-maintenance/SKILL.md`, and `.opencode/skills/README.md`. Correct stale “todo/implementation pending” references for metadata command handlers and document the new canonical storage contract.
  **Must NOT do**: Do not invent new docs hierarchy. Do not leave stale claims like “handlers are todo stubs” or `.novelsaga/cache/index` as active guidance. Do not modify unrelated skill/command semantics.

  **Recommended Agent Profile**:
  - Category: `writing` — Reason: concentrated documentation synchronization.
  - Skills: [`docs-maintenance`, `project-architecture`, `cli-dev`] — docs maintenance plus exact CLI/storage contract wording.
  - Omitted: [`lsp-dev`] — no protocol logic changes expected.

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: [] | Blocked By: [12, 13]

  **References** (executor has NO interview context — be exhaustive):
  - `AGENTS.md:25-63,82-122` — root commands/conventions/progress docs.
  - `projects/cli/AGENTS.md:277-297` — stale metadata handler status.
  - `.opencode/command/sync-docs.md:58-80,102-135` — doc sync guidance.
  - `.opencode/skills/docs-maintenance/SKILL.md:15-30,80-129` — docs-maintenance rules.
  - `.opencode/skills/README.md:9-23,64-94` — skill index and doc references.
  - `.sisyphus/roadmaps/p2-metadata-manager-breakdown.md:83-87,164-167` and `.sisyphus/roadmaps/metadata-storage-risk-register.md:14-39` — storage/doc targets.

  **Acceptance Criteria** (agent-executable only):
  - [x] All named docs reference `<workspace>/.cache/novelsaga/sled` as current behavior, or clearly mark prior behavior as historical only.
  - [x] `projects/cli/AGENTS.md` no longer says metadata handlers are `todo!()` stubs.
  - [x] `.opencode` docs mention updated storage/doc-sync contract where relevant.
  - [x] No active doc presents global metadata cache fallback as current behavior.

  **QA Scenarios** (MANDATORY — task incomplete without these):

  ```
  Scenario: Documentation contract grep passes
    Tool: Bash
    Steps: Run `rg -n '\.novelsaga/cache/index|cache_dir\(\)\.join\("metadata"\)|Handlers are currently `todo!\(\)` stubs|implementation pending' AGENTS.md projects .sisyphus .opencode`
    Expected: Remaining matches are only explicitly marked historical/stale references; active guidance is updated.
    Evidence: .sisyphus/evidence/task-14-docs-grep.log

  Scenario: Named docs all show canonical path
    Tool: Read
    Steps: Open each required doc and verify canonical storage contract wording.
    Expected: Every active doc uses `<workspace>/.cache/novelsaga/sled` as the current contract.
    Evidence: .sisyphus/evidence/task-14-docs-read.txt
  ```

  **Commit**: YES | Message: `docs(metadata): synchronize storage contract guidance` | Files: [`.sisyphus/roadmaps/*.md`, `.sisyphus/plans/archive/metadata-completion-plan.md`, `**/AGENTS.md`, `.opencode/command/*.md`, `.opencode/skills/**/*.md`]

## Final Verification Wave (4 parallel agents, ALL must APPROVE)

- [x] F1. Plan Compliance Audit — oracle
- [x] F2. Code Quality Review — unspecified-high
- [x] F3. Real Manual QA — unspecified-high
- [x] F4. Scope Fidelity Check — deep

## Commit Strategy

- Major Step 1 commit: `refactor(metadata): establish canonical storage contract`
- Major Step 2 commit: `fix(metadata): migrate cli and lsp to shared resolver`
- Major Step 3 commit: `docs(metadata): synchronize storage contract guidance`
- Pre-commit gate before every major-step commit:
  - `cargo test -p novelsaga-cli`
  - `cargo test -p novelsaga-core`
  - `cargo clippy -p novelsaga-cli --all-targets -- -D warnings`
  - `cargo clippy -p novelsaga-core --all-targets -- -D warnings`
  - `pnpm exec eslint .`
- Oracle review follows each major-step commit immediately.
- If Oracle rejects for structural reasons (duplicate source of truth, ambiguous fallback, partial migration, stale docs), reset to the previous accepted commit and rewrite the whole major step; do not patch forward.

## Success Criteria

- `index`, `list`, `show`, LSP startup/open/update, and LSP execute-command metadata routes all use one canonical workspace-local DB location.
- Legacy `.novelsaga/cache/index` data is deterministically migrated or surfaced according to the defined compatibility rule; no implicit global cache survives.
- `WorkspaceConfig` and documentation surfaces match actual runtime behavior.
- Stage commits pass hooks/linters/tests and clear Oracle review without structural rejection.
