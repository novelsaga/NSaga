# CLI Format Subcommand Implementation (Plan 4)

## TL;DR

> **Quick Summary**: 实现 `format` 子命令，用于格式化 NovelSaga 配置文件。
>
> **Deliverables**:
>
> - 配置文件格式化逻辑
> - `--check` 模式（只检查不修改）
> - 支持指定文件或自动检测
>
> **Estimated Effort**: Medium (2-4 hours)
> **Parallel Execution**: NO
> **Depends On**: Plan 1 (cli-subcommand-base) 必须先完成

---

## Context

### Original Request

用户希望 `format` 子命令能够格式化配置文件。

### Prerequisites

- **Plan 1 必须完成**: `Commands::Format { files, check }` 变体已存在（带 todo!）

### Current State (After Plan 1)

```rust
Some(Commands::Format { files, check }) => {
    todo!("Format command not implemented yet. Files: {:?}, Check: {}", files, check);
}
```

---

## Work Objectives

### Core Objective

实现配置文件格式化功能，支持检查模式和自动修复模式。

### Concrete Deliverables

- `projects/cli/src/commands/format.rs` - format 命令实现
- 配置文件解析和重新序列化逻辑

### Definition of Done

- [ ] `novelsaga format` 格式化当前目录配置
- [ ] `novelsaga format file.yaml` 格式化指定文件
- [ ] `novelsaga format --check` 只检查，不修改
- [ ] 返回正确的 exit code

### Must Have

- 支持 YAML 配置文件格式化
- `--check` 模式返回非零 exit code 如果需要格式化
- 友好的输出消息

### Must NOT Have (Guardrails)

- ❌ 支持非配置文件格式
- ❌ 复杂的格式化选项（缩进大小等）
- ❌ 与编辑器集成

---

## Verification Strategy

### Agent-Executed QA Scenarios (MANDATORY)

见各 TODO 任务。

---

## TODOs

> **NOTE**: 此计划需要进一步细化。

- [ ] 1. Implement format command logic

  **What to do**:
  1. 创建 `projects/cli/src/commands/format.rs`
  2. 实现文件发现逻辑（无参数时自动检测）
  3. 实现 YAML 格式化（解析 + 重新序列化）

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: [`rust-dev`]

  **References**:
  - serde_yaml 格式化文档

  **Acceptance Criteria**:

  ```
  Scenario: Format command formats file
    Tool: Bash
    Steps:
      1. echo "key:value" > /tmp/test.yaml  # 无空格
      2. cargo run -p novelsaga-cli -- format /tmp/test.yaml
      3. cat /tmp/test.yaml
      4. Assert: file contains "key: value" (with space)
    Expected Result: File formatted
    Evidence: File content captured

  Scenario: Check mode detects unformatted file
    Tool: Bash
    Steps:
      1. echo "key:value" > /tmp/test2.yaml
      2. cargo run -p novelsaga-cli -- format --check /tmp/test2.yaml; echo "exit: $?"
      3. Assert: exit code is non-zero
    Expected Result: Non-zero exit for unformatted
    Evidence: Exit code captured

  Scenario: Check mode passes for formatted file
    Tool: Bash
    Steps:
      1. cargo run -p novelsaga-cli -- format /tmp/test2.yaml  # format first
      2. cargo run -p novelsaga-cli -- format --check /tmp/test2.yaml; echo "exit: $?"
      3. Assert: exit code is 0
    Expected Result: Zero exit for formatted
    Evidence: Exit code captured
  ```

  **Commit**: YES
  - Message: `feat(cli): implement format subcommand for config file formatting`

---

## Success Criteria

### Verification Commands

```bash
# 格式化文件
novelsaga format config.yaml

# 检查模式
novelsaga format --check config.yaml
echo $?  # 0 if formatted, non-zero otherwise

# 自动检测
novelsaga format  # Formats all config files in current dir
```

### Final Checklist

- [ ] `novelsaga format` 格式化配置文件
- [ ] `novelsaga format --check` 只检查
- [ ] 正确的 exit code
- [ ] 支持多文件参数

---

## Notes

此计划为框架性计划，需要确认：

1. 支持的配置文件格式（YAML/TOML/JSON）
2. 格式化规则（缩进、排序等）
3. 文件自动发现规则
