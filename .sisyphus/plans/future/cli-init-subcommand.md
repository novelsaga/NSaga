# CLI Init Subcommand Implementation (Plan 3)

## TL;DR

> **Quick Summary**: 实现 `init` 子命令，用于初始化新的 NovelSaga 项目结构。
>
> **Deliverables**:
>
> - 项目初始化逻辑模块
> - 默认配置文件模板
> - 目录结构创建
>
> **Estimated Effort**: Medium (2-4 hours)
> **Parallel Execution**: NO
> **Depends On**: Plan 1 (cli-subcommand-base) 必须先完成

---

## Context

### Original Request

用户希望 `init` 子命令能够初始化新的 NovelSaga 项目。

### Prerequisites

- **Plan 1 必须完成**: `Commands::Init { path }` 变体已存在（带 todo!）

### Current State (After Plan 1)

```rust
Some(Commands::Init { path }) => {
    todo!("Init command not implemented yet. Path: {:?}", path);
}
```

---

## Work Objectives

### Core Objective

实现项目初始化功能，创建标准 NovelSaga 项目结构。

### Concrete Deliverables

- `projects/cli/src/commands/mod.rs` - 命令实现模块
- `projects/cli/src/commands/init.rs` - init 命令实现
- 默认配置文件模板（嵌入或资源文件）

### Definition of Done

- [ ] `novelsaga init` 在当前目录创建项目结构
- [ ] `novelsaga init ./my-novel` 在指定目录创建
- [ ] 已存在项目时给出警告/错误
- [ ] 创建基本配置文件

### Must Have

- 创建 `novelsaga.yaml` 或类似配置文件
- 检查目录是否已初始化
- 友好的成功/错误消息

### Must NOT Have (Guardrails)

- ❌ 复杂的模板系统
- ❌ 交互式向导（保持简单）
- ❌ 网络请求（离线可用）

---

## Verification Strategy

### Agent-Executed QA Scenarios (MANDATORY)

见各 TODO 任务。

---

## TODOs

> **NOTE**: 此计划需要进一步细化。以下是初步框架。

- [ ] 1. Create commands module structure

  **What to do**:
  1. 创建 `projects/cli/src/commands/mod.rs`
  2. 创建 `projects/cli/src/commands/init.rs`
  3. 在 `main.rs` 中引入并调用

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: [`rust-dev`]

  **References**:
  - `projects/cli/src/main.rs` - 命令分发位置

  **Acceptance Criteria**:

  ```
  Scenario: Init command creates config file
    Tool: Bash
    Steps:
      1. mkdir -p /tmp/test-init && cd /tmp/test-init
      2. cargo run -p novelsaga-cli -- init 2>&1
      3. ls -la /tmp/test-init
      4. Assert: novelsaga.yaml or similar config exists
    Expected Result: Config file created
    Evidence: Directory listing captured
  ```

  **Commit**: YES
  - Message: `feat(cli): implement init subcommand for project initialization`

---

- [ ] 2. Add project structure templates

  **What to do**:
  1. 定义默认配置文件内容
  2. 可选：创建示例章节目录结构

  **Acceptance Criteria**:

  ```
  Scenario: Init in existing project warns user
    Tool: Bash
    Steps:
      1. cd /tmp/test-init (already initialized)
      2. cargo run -p novelsaga-cli -- init 2>&1
      3. Assert: output contains "already" or "exists"
    Expected Result: Warning shown
    Evidence: Output captured
  ```

---

## Success Criteria

### Verification Commands

```bash
# 新目录初始化
mkdir /tmp/new-novel && cd /tmp/new-novel
novelsaga init
ls -la  # Should show novelsaga.yaml

# 指定路径初始化
novelsaga init /tmp/another-novel
ls /tmp/another-novel

# 已存在项目
novelsaga init  # Should warn
```

### Final Checklist

- [ ] `novelsaga init` 创建配置文件
- [ ] `novelsaga init <path>` 支持指定路径
- [ ] 已初始化目录给出提示
- [ ] 无网络依赖

---

## Notes

此计划为框架性计划，具体实现细节（配置文件格式、目录结构）需要根据 NovelSaga 的实际需求进一步细化。建议在执行前与用户确认：

1. 配置文件格式（YAML/TOML/JSON）
2. 默认目录结构
3. 是否需要 .gitignore 等辅助文件
