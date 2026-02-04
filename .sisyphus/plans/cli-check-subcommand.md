# CLI Check Subcommand Implementation (Plan 5)

## TL;DR

> **Quick Summary**: 实现 `check` 子命令，用于验证 NovelSaga 配置文件的语法和有效性。
> 
> **Deliverables**:
> - 配置文件验证逻辑
> - 错误报告格式化
> - 支持指定文件或自动检测
> 
> **Estimated Effort**: Medium (2-4 hours)
> **Parallel Execution**: NO
> **Depends On**: Plan 1 (cli-subcommand-base) 必须先完成

---

## Context

### Original Request
用户希望 `check` 子命令能够验证配置文件。

### Prerequisites
- **Plan 1 必须完成**: `Commands::Check { files }` 变体已存在（带 todo!）

### Current State (After Plan 1)
```rust
Some(Commands::Check { files }) => {
    todo!("Check command not implemented yet. Files: {:?}", files);
}
```

---

## Work Objectives

### Core Objective
实现配置文件验证功能，检查语法错误和逻辑错误。

### Concrete Deliverables
- `projects/cli/src/commands/check.rs` - check 命令实现
- 错误诊断和报告逻辑

### Definition of Done
- [ ] `novelsaga check` 检查当前目录配置
- [ ] `novelsaga check file.yaml` 检查指定文件
- [ ] 语法错误报告行号和位置
- [ ] 返回正确的 exit code

### Must Have
- 语法验证（YAML 解析）
- Schema 验证（配置结构）
- 友好的错误消息（行号、列号）
- 正确的 exit code（0=通过, 1=失败）

### Must NOT Have (Guardrails)
- ❌ 自动修复功能（那是 format 的职责）
- ❌ 复杂的诊断系统
- ❌ LSP 风格的增量检查

---

## Verification Strategy

### Agent-Executed QA Scenarios (MANDATORY)

见各 TODO 任务。

---

## TODOs

> **NOTE**: 此计划需要进一步细化。

- [ ] 1. Implement check command logic

  **What to do**:
  1. 创建 `projects/cli/src/commands/check.rs`
  2. 实现文件发现逻辑
  3. 实现 YAML 语法验证
  4. 实现配置 Schema 验证

  **Recommended Agent Profile**:
  - **Category**: `unspecified-low`
  - **Skills**: [`rust-dev`]

  **References**:
  - `projects/core/src/config/` - 配置类型定义
  - serde_yaml 错误处理

  **Acceptance Criteria**:

  ```
  Scenario: Check passes for valid config
    Tool: Bash
    Steps:
      1. Create valid config file
      2. cargo run -p novelsaga-cli -- check /tmp/valid.yaml; echo "exit: $?"
      3. Assert: exit code is 0
      4. Assert: output contains "ok" or "valid" or no errors
    Expected Result: Check passes
    Evidence: Output captured

  Scenario: Check fails for invalid YAML syntax
    Tool: Bash
    Steps:
      1. echo "key: [unclosed" > /tmp/invalid.yaml
      2. cargo run -p novelsaga-cli -- check /tmp/invalid.yaml 2>&1; echo "exit: $?"
      3. Assert: exit code is non-zero
      4. Assert: output contains error message with line info
    Expected Result: Syntax error reported
    Evidence: Error output captured

  Scenario: Check fails for invalid config schema
    Tool: Bash
    Steps:
      1. echo "unknown_field: value" > /tmp/schema-invalid.yaml
      2. cargo run -p novelsaga-cli -- check /tmp/schema-invalid.yaml 2>&1; echo "exit: $?"
      3. Assert: exit code is non-zero (or warning)
    Expected Result: Schema error reported
    Evidence: Error output captured

  Scenario: Check multiple files
    Tool: Bash
    Steps:
      1. cargo run -p novelsaga-cli -- check /tmp/valid.yaml /tmp/invalid.yaml 2>&1
      2. Assert: reports both files
    Expected Result: Multiple files checked
    Evidence: Output captured
  ```

  **Commit**: YES
  - Message: `feat(cli): implement check subcommand for config validation`

---

## Success Criteria

### Verification Commands
```bash
# 检查单个文件
novelsaga check config.yaml
echo $?  # 0 if valid

# 检查多个文件
novelsaga check config.yaml chapter1.yaml

# 自动检测
novelsaga check  # Checks all config files
```

### Final Checklist
- [ ] `novelsaga check` 验证配置文件
- [ ] 语法错误报告行号
- [ ] Schema 错误报告字段
- [ ] 正确的 exit code
- [ ] 支持多文件参数

---

## Notes

此计划为框架性计划，需要确认：

1. 支持的配置文件格式
2. Schema 定义位置（Core 中的类型）
3. 错误报告格式（简单文本 vs JSON vs 诊断格式）
4. 与 LSP 诊断的关系（复用逻辑？）

---

## Integration with Core

`check` 命令应该复用 Core 中的配置解析逻辑：

```rust
// 可能的实现模式
use novelsaga_core::config::Config;

fn check_file(path: &Path) -> Result<(), CheckError> {
    let content = fs::read_to_string(path)?;
    let _config: Config = serde_yaml::from_str(&content)?;
    Ok(())
}
```

这确保 CLI check 和 Core 使用相同的验证逻辑。
