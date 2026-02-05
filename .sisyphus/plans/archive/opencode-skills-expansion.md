# NovelSaga OpenCode Skills 扩展计划 [COMPLETED 2026-02-06]

> **归档**: 成功将 NovelSaga 的 OpenCode skills 从 3 个扩展到 13 个
> **实际成果**: 创建了 10 个新 skills，更新了 skills 索引和根文档

## TL;DR

> **目标**: 将 NovelSaga 的 OpenCode skills 从 3 个扩展到 11 个，覆盖所有开发场景
>
> **新增 Skills**: 8 个（拆分 4 个 + 新增 7 个）
>
> - 拆分: core-dev, cli-dev, nix-env, nix-build
> - 新增: xtask-build, testing-guide, lsp-dev, ffi-diplomat, end-to-end-workflow, project-architecture, docs-maintenance
>
> **删除**: 2 个 command（/sync-docs 等将被 skills 替代）
>
> **预计工作量**: Medium（约 2-3 小时）
> **并行执行**: YES - 4 个 waves
> **关键路径**: core-dev/cli-dev → lsp-dev → end-to-end-workflow

---

## Context

### 原始需求

用户希望为 NovelSaga 项目添加更多 OpenCode skills，目前只有 3 个感觉不够用。

### 面试总结

**用户工作范围**: Rust core、CLI/LSP、TypeScript bridges、构建发布、FFI 全覆盖

**痛点**:

- LSP 协议实现细节
- FFI/Diplomat 绑定
- xtask 构建脚本
- 测试策略
- 配置系统

**新成员需求**: 端到端流程指导、架构理解

**特殊需求**: AI 文档自动检测更新 + 归档已完成 plans

### 关键决策

1. **只保留 skill，删除 command** - 全力完善 skills 体系
2. **AI 行动导向** - skills 指导 AI 如何行动，详细文档保留在 AGENTS.md
3. **组合多个 skills** - 添加功能时加载多个相关 skills
4. **拆分现有 skills** - rust-dev → core-dev + cli-dev, nix-workflow → nix-env + nix-build

---

## Work Objectives

### Core Objective

创建 11 个完整的 OpenCode skills，覆盖 NovelSaga 项目的所有开发场景，使 AI agents 能够在任何任务中获得准确的上下文和指导。

### Concrete Deliverables

- 8 个新的 skill 文件（`.opencode/skills/*/SKILL.md`）
- 更新的现有 skill 文件（如有必要）
- 删除冗余的 command 文件
- 更新的 `.opencode/skills/README.md`（skill 索引和快速选择指南）

### Definition of Done

- [ ] 所有 11 个 skills 创建完成并通过验证
- [ ] 每个 skill 包含 Quick Reference、Anti-Patterns、When to Use
- [ ] 所有文件引用指向真实存在的代码文件
- [ ] `.opencode/skills/README.md` 更新完成
- [ ] 冗余 commands 已删除

### Must Have

- 所有 skills 遵循统一格式（参考现有 3 个 skills）
- 每个 skill 有明确的边界，不与其他 skills 重叠
- 包含具体的代码示例和文件引用
- 包含至少 3 个 anti-patterns

### Must NOT Have (Guardrails)

- 不重复 AGENTS.md 的内容（引用而非复制）
- 不包含过时的代码示例
- 不创建与其他 skills 功能重叠的内容
- 不包含通用教程（只包含 NovelSaga 特有的内容）

---

## Verification Strategy

### Test Decision

- **Infrastructure exists**: NO（需要创建新的 skill 文件）
- **Automated tests**: NO（skills 是文档，人工验证）
- **Framework**: N/A

### Agent-Executed QA Scenarios (MANDATORY)

每个 skill 创建后，执行以下验证：

```
Scenario: Skill file structure validation
  Tool: Bash
  Preconditions: Skill file created at .opencode/skills/{name}/SKILL.md
  Steps:
    1. cat .opencode/skills/{name}/SKILL.md
    2. Assert: File contains "---" frontmatter with name and description
    3. Assert: File contains "## Quick Reference" section
    4. Assert: File contains "## Anti-Patterns" section with at least 3 items
    5. Assert: File contains "## When to Use" section
  Expected Result: Skill file structure完整
  Evidence: Terminal output

Scenario: Code references validation
  Tool: Bash
  Preconditions: Skill file references code files
  Steps:
    1. grep -oE 'projects/[a-zA-Z0-9/_-]+\.(rs|ts|nix)' .opencode/skills/{name}/SKILL.md
    2. For each referenced file: test -f {file} && echo "EXISTS" || echo "MISSING"
    3. Assert: All referenced files exist
  Expected Result: 所有代码引用有效
  Evidence: Terminal output

Scenario: Skill loading test
  Tool: Bash
  Preconditions: OpenCode environment available
  Steps:
    1. echo "Testing skill: {name}"
    2. Verify skill appears in OpenCode skill list
    3. Verify skill description is accurate
  Expected Result: Skill 可被正确加载
  Evidence: Skill list output
```

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (基础拆分 - 可并行):
├── Task 1: core-dev (从 rust-dev 拆分)
├── Task 2: cli-dev (从 rust-dev 拆分)
├── Task 3: nix-env (从 nix-workflow 拆分)
└── Task 4: nix-build (从 nix-workflow 拆分)

Wave 2 (核心功能 - 可并行):
├── Task 5: testing-guide
├── Task 6: lsp-dev
└── Task 7: ffi-diplomat

Wave 3 (综合指导 - 依赖 Wave 1-2):
├── Task 8: project-architecture
└── Task 9: end-to-end-workflow

Wave 4 (维护工具 - 可并行):
├── Task 10: docs-maintenance
└── Task 11: 更新索引和清理

Critical Path: Wave 1 → Wave 2 → Wave 3
Parallel Speedup: ~60% faster than sequential
```

### Dependency Matrix

| Task             | Depends On          | Blocks | Can Parallelize With |
| ---------------- | ------------------- | ------ | -------------------- |
| 1 (core-dev)     | None                | None   | 2, 3, 4              |
| 2 (cli-dev)      | None                | None   | 1, 3, 4              |
| 3 (nix-env)      | None                | None   | 1, 2, 4              |
| 4 (nix-build)    | None                | None   | 1, 2, 3              |
| 5 (testing)      | None                | None   | 6, 7                 |
| 6 (lsp-dev)      | 2 (cli-dev)         | 9      | 5, 7                 |
| 7 (ffi-diplomat) | 1 (core-dev)        | 9      | 5, 6                 |
| 8 (architecture) | 1, 2, 3, 4, 5, 6, 7 | 9      | None                 |
| 9 (workflow)     | 6, 7, 8             | None   | None                 |
| 10 (docs-maint)  | None                | None   | 8, 9                 |
| 11 (索引)        | All                 | None   | None                 |

---

## TODOs

- [ ] 1. core-dev - Rust Core 库开发

  **What to do**:
  - 从现有 rust-dev 提取 core 相关内容
  - 创建 `.opencode/skills/core-dev/SKILL.md`
  - 包含: State 管理 (Initializer)、类型导出 (ts-rs)、核心库模式

  **Must NOT do**:
  - 不包含 CLI/LSP 相关内容（移到 cli-dev）
  - 不包含通用 Rust 教程

  **Recommended Agent Profile**:
  - **Category**: `unspecified-medium`
  - **Skills**: [`rust-dev`]
  - **Skills Evaluated but Omitted**:
    - `ts-bridge`: 不涉及 bridge 开发
    - `nix-workflow`: 不涉及构建

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: ffi-diplomat
  - **Blocked By**: None

  **References**:
  - `projects/core/src/state/init.rs` - Initializer 模式
  - `projects/core/src/config/` - 配置系统
  - `projects/core/AGENTS.md` - 详细文档

  **Acceptance Criteria**:
  - [ ] Skill file created at `.opencode/skills/core-dev/SKILL.md`
  - [ ] Contains Quick Reference section
  - [ ] Contains at least 3 anti-patterns
  - [ ] All code references valid
  - [ ] "When to Use" clearly differentiates from cli-dev

  **Agent-Executed QA**:

  ```
  Scenario: core-dev skill validation
    Tool: Bash
    Steps:
      1. test -f .opencode/skills/core-dev/SKILL.md
      2. grep -q "## Quick Reference" .opencode/skills/core-dev/SKILL.md
      3. grep -c "## Anti-Patterns" .opencode/skills/core-dev/SKILL.md | grep -q "1"
      4. grep "projects/core" .opencode/skills/core-dev/SKILL.md | while read f; do test -f "$f" && echo "OK: $f"; done
    Expected: All checks pass
  ```

  **Commit**: YES
  - Message: `feat(skills): add core-dev skill for Rust core development`
  - Files: `.opencode/skills/core-dev/SKILL.md`

---

- [ ] 2. cli-dev - CLI 和 LSP Server 开发

  **What to do**:
  - 从现有 rust-dev 提取 CLI 相关内容
  - 创建 `.opencode/skills/cli-dev/SKILL.md`
  - 包含: CLI 结构、命令处理、bridge 管理

  **Must NOT do**:
  - 不包含 LSP 协议细节（移到 lsp-dev）
  - 不包含 core 库内容

  **Recommended Agent Profile**:
  - **Category**: `unspecified-medium`
  - **Skills**: [`rust-dev`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: lsp-dev
  - **Blocked By**: None

  **References**:
  - `projects/cli/src/main.rs` - CLI 入口
  - `projects/cli/src/bridge/` - Bridge 管理
  - `projects/cli/AGENTS.md` - 详细文档

  **Acceptance Criteria**:
  - [ ] Skill file created
  - [ ] Contains CLI-specific patterns
  - [ ] References bridge management code
  - [ ] "When to Use" differentiates from core-dev and lsp-dev

  **Commit**: YES
  - Message: `feat(skills): add cli-dev skill for CLI development`

---

- [ ] 3. nix-env - Nix 开发环境

  **What to do**:
  - 从现有 nix-workflow 提取环境相关内容
  - 创建 `.opencode/skills/nix-env/SKILL.md`
  - 包含: direnv 设置、开发工具链、环境故障排除

  **Must NOT do**:
  - 不包含构建命令（移到 nix-build）

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: [`nix-workflow`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1

  **References**:
  - `flake.nix` - 环境定义
  - `.envrc` - direnv 配置

  **Acceptance Criteria**:
  - [ ] Skill file created
  - [ ] Contains environment setup steps
  - [ ] Contains troubleshooting guide

  **Commit**: YES
  - Message: `feat(skills): add nix-env skill for development environment`

---

- [ ] 4. nix-build - Nix 构建系统

  **What to do**:
  - 从现有 nix-workflow 提取构建相关内容
  - 创建 `.opencode/skills/nix-build/SKILL.md`
  - 包含: 构建命令、hash 更新、多平台构建

  **Must NOT do**:
  - 不包含环境设置（移到 nix-env）

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: [`nix-workflow`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1

  **References**:
  - `nix/packages.nix` - 包定义
  - `.github/workflows/` - CI 构建

  **Acceptance Criteria**:
  - [ ] Skill file created
  - [ ] Contains build commands table
  - [ ] Contains hash update procedure

  **Commit**: YES
  - Message: `feat(skills): add nix-build skill for Nix builds`

---

- [ ] 5. testing-guide - 测试指南

  **What to do**:
  - 创建 `.opencode/skills/testing-guide/SKILL.md`
  - **窄边界**: 只教 NovelSaga 特有模式
  - 包含: same-file tests、E2E 测试结构、测试矩阵

  **Must NOT do**:
  - 不包含 cargo test 基础教程
  - 不包含通用测试理论

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: [`rust-dev`, `ts-bridge`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2

  **References**:
  - `projects/core/src/**/*.rs` - 查找 #[cfg(test)] 示例
  - `xtask/src/tasks/test/` - E2E 测试
  - `cargo test` 输出

  **Acceptance Criteria**:
  - [ ] Skill file created
  - [ ] Documents same-file test pattern
  - [ ] Documents E2E test structure
  - [ ] Contains test matrix (runtimes × formats)

  **Commit**: YES
  - Message: `feat(skills): add testing-guide for NovelSaga testing patterns`

---

- [ ] 6. lsp-dev - LSP 功能开发

  **What to do**:
  - 创建 `.opencode/skills/lsp-dev/SKILL.md`
  - **窄边界**: 只教 LSP 特有内容
  - 包含: tower_lsp Backend、Capability 注册、Document sync

  **Must NOT do**:
  - 不包含完整功能流程（依赖 end-to-end-workflow）
  - 不包含通用 LSP 协议文档

  **Recommended Agent Profile**:
  - **Category**: `unspecified-medium`
  - **Skills**: [`rust-dev`, `cli-dev`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with testing-guide, ffi-diplomat)
  - **Parallel Group**: Wave 2
  - **Blocks**: end-to-end-workflow
  - **Blocked By**: cli-dev

  **References**:
  - `projects/cli/src/lsp/` - LSP 实现
  - `tower_lsp` crate 文档

  **Acceptance Criteria**:
  - [ ] Skill file created
  - [ ] Documents Backend trait implementation
  - [ ] Documents Capability registration
  - [ ] Documents Document sync modes

  **Commit**: YES
  - Message: `feat(skills): add lsp-dev for LSP protocol implementation`

---

- [ ] 7. ffi-diplomat - FFI 绑定开发

  **What to do**:
  - 创建 `.opencode/skills/ffi-diplomat/SKILL.md`
  - **窄边界**: 只教 diplomat 语法
  - 包含: #[diplomat::bridge]、opaque types、方法暴露

  **Must NOT do**:
  - 不包含完整跨平台流程（依赖 end-to-end-workflow）
  - 不包含 iOS/Android 集成细节

  **Recommended Agent Profile**:
  - **Category**: `unspecified-medium`
  - **Skills**: [`rust-dev`, `core-dev`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 2
  - **Blocks**: end-to-end-workflow
  - **Blocked By**: core-dev

  **References**:
  - `projects/core/src/diplomat_ffi.rs` - FFI 绑定
  - `diplomat` crate 文档

  **Acceptance Criteria**:
  - [ ] Skill file created
  - [ ] Documents diplomat::bridge pattern
  - [ ] Documents opaque type pattern
  - [ ] Documents constructor/method exposure

  **Commit**: YES
  - Message: `feat(skills): add ffi-diplomat for FFI bindings`

---

- [ ] 8. project-architecture - 项目架构

  **What to do**:
  - 创建 `.opencode/skills/project-architecture/SKILL.md`
  - **窄边界**: 只教模块关系图
  - 包含: 模块依赖图、数据流、状态初始化序列

  **Must NOT do**:
  - 不包含设计决策历史
  - 不包含 API 文档

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: [] (综合 skill)

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocks**: end-to-end-workflow
  - **Blocked By**: Wave 1 + Wave 2

  **References**:
  - `AGENTS.md` - 架构概述
  - `projects/*/AGENTS.md` - 模块详情

  **Acceptance Criteria**:
  - [ ] Skill file created
  - [ ] Contains module dependency graph
  - [ ] Contains data flow diagram
  - [ ] Contains state initialization sequence

  **Commit**: YES
  - Message: `feat(skills): add project-architecture skill`

---

- [ ] 9. end-to-end-workflow - 端到端工作流

  **What to do**:
  - 创建 `.opencode/skills/end-to-end-workflow/SKILL.md`
  - **窄边界**: 只教改哪些文件（决策树）
  - 包含: 功能类型 → 文件映射、类型导出流程、构建顺序

  **Must NOT do**:
  - 不包含模块具体实现细节（引用其他 skills）
  - 不包含发布流程

  **Recommended Agent Profile**:
  - **Category**: `unspecified-medium`
  - **Skills**: [] (引用所有其他 skills)

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3
  - **Blocked By**: lsp-dev, ffi-diplomat, project-architecture

  **References**:
  - 所有其他 skills
  - 示例 feature 实现（从 git history 找）

  **Acceptance Criteria**:
  - [ ] Skill file created
  - [ ] Contains decision tree for feature types
  - [ ] Documents type export workflow
  - [ ] Documents build order

  **Commit**: YES
  - Message: `feat(skills): add end-to-end-workflow skill`

---

- [ ] 10. docs-maintenance - 文档维护

  **What to do**:
  - 创建 `.opencode/skills/docs-maintenance/SKILL.md`
  - **宽边界**: 教何时更新 + 怎么写 + 模板
  - 包含: 更新触发矩阵、同步流程、plan 归档、文档模板

  **Must NOT do**:
  - 不替代 /sync-docs 命令（skill 指导 AI，命令执行操作）

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 4

  **References**:
  - `.sisyphus/` 目录结构
  - `AGENTS.md` 文件
  - `.opencode/skills/` 目录

  **Acceptance Criteria**:
  - [ ] Skill file created
  - [ ] Contains update trigger matrix
  - [ ] Contains sync procedures
  - [ ] Contains plan archiving checklist
  - [ ] Contains document templates

  **Commit**: YES
  - Message: `feat(skills): add docs-maintenance skill`

---

- [ ] 11. 更新索引和清理

  **What to do**:
  - 更新 `.opencode/skills/README.md`（创建如果不存在）
  - 添加 skill 快速选择指南
  - 删除冗余的 command 文件（/sync-docs 等）
  - 更新根目录 AGENTS.md 中的 skills 引用

  **Must NOT do**:
  - 不删除用户可能依赖的其他 commands

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO（最后执行）
  - **Parallel Group**: Wave 4
  - **Blocked By**: All other tasks

  **References**:
  - `.opencode/skills/*/SKILL.md` - 所有 skills
  - `.opencode/commands/` - 需要删除的 commands

  **Acceptance Criteria**:
  - [ ] `.opencode/skills/README.md` created/updated
  - [ ] README contains skill selection guide
  - [ ] Redundant commands removed
  - [ ] AGENTS.md updated with new skills reference

  **Agent-Executed QA**:

  ```
  Scenario: Skills index validation
    Tool: Bash
    Steps:
      1. test -f .opencode/skills/README.md
      2. ls .opencode/skills/*/SKILL.md | wc -l | grep -q "11"
      3. grep -q "core-dev" .opencode/skills/README.md
      4. grep -q "cli-dev" .opencode/skills/README.md
      5. grep -q "lsp-dev" .opencode/skills/README.md
    Expected: Index complete and accurate
  ```

  **Commit**: YES
  - Message: `docs(skills): add skills index and remove redundant commands`

---

## Commit Strategy

| After Task | Message                                                          | Files                                               |
| ---------- | ---------------------------------------------------------------- | --------------------------------------------------- |
| 1          | `feat(skills): add core-dev skill for Rust core development`     | `.opencode/skills/core-dev/SKILL.md`                |
| 2          | `feat(skills): add cli-dev skill for CLI development`            | `.opencode/skills/cli-dev/SKILL.md`                 |
| 3          | `feat(skills): add nix-env skill for development environment`    | `.opencode/skills/nix-env/SKILL.md`                 |
| 4          | `feat(skills): add nix-build skill for Nix builds`               | `.opencode/skills/nix-build/SKILL.md`               |
| 5          | `feat(skills): add testing-guide for NovelSaga testing patterns` | `.opencode/skills/testing-guide/SKILL.md`           |
| 6          | `feat(skills): add lsp-dev for LSP protocol implementation`      | `.opencode/skills/lsp-dev/SKILL.md`                 |
| 7          | `feat(skills): add ffi-diplomat for FFI bindings`                | `.opencode/skills/ffi-diplomat/SKILL.md`            |
| 8          | `feat(skills): add project-architecture skill`                   | `.opencode/skills/project-architecture/SKILL.md`    |
| 9          | `feat(skills): add end-to-end-workflow skill`                    | `.opencode/skills/end-to-end-workflow/SKILL.md`     |
| 10         | `feat(skills): add docs-maintenance skill`                       | `.opencode/skills/docs-maintenance/SKILL.md`        |
| 11         | `docs(skills): add skills index and remove redundant commands`   | `.opencode/skills/README.md`, `.opencode/commands/` |

---

## Success Criteria

### Verification Commands

```bash
# Count skills
ls .opencode/skills/*/SKILL.md | wc -l  # Expected: 11

# Verify structure
for f in .opencode/skills/*/SKILL.md; do
  echo "Checking $f..."
  grep -q "^---" "$f" && echo "  ✓ Frontmatter" || echo "  ✗ Missing frontmatter"
  grep -q "## Quick Reference" "$f" && echo "  ✓ Quick Reference" || echo "  ✗ Missing Quick Reference"
  grep -q "## Anti-Patterns" "$f" && echo "  ✓ Anti-Patterns" || echo "  ✗ Missing Anti-Patterns"
  grep -q "## When to Use" "$f" && echo "  ✓ When to Use" || echo "  ✗ Missing When to Use"
done

# Check index exists
test -f .opencode/skills/README.md && echo "✓ Index exists" || echo "✗ Index missing"
```

### Final Checklist

- [ ] 11 skill files created
- [ ] All skills have consistent structure
- [ ] All code references valid
- [ ] Skills index created
- [ ] Redundant commands removed
- [ ] AGENTS.md updated
