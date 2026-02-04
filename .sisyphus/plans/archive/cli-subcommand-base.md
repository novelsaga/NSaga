# CLI Subcommand System - Base Framework

## TL;DR

> **Quick Summary**: Refactor CLI from flat `--lsp` flag to proper subcommand architecture with `lsp`, `init`, `format`, `check` subcommands while preserving global runtime arguments.
>
> **Deliverables**:
>
> - Modified `args/mod.rs` with `Commands` enum and subcommand support
> - Modified `main.rs` with command dispatch and status display
> - Unit tests for argument parsing
>
> **Estimated Effort**: Short (1-2 hours)
> **Parallel Execution**: NO - sequential (2 files, tightly coupled)
> **Critical Path**: Task 1 (args) → Task 2 (main) → Task 3 (tests)

---

## Context

### Original Request

用户希望对CLI的args系统进行重构：

- 添加子命令系统（`--lsp` → `lsp` 子命令）
- 保留全局参数系统（runtime相关）
- 子命令可以有独立参数

### Interview Summary

**Key Discussions**:

- 默认行为：无子命令时显示完整诊断信息（版本+runtime+config+help提示）
- 向后兼容：直接破坏性变更，不保留 `--lsp` 别名
- 子命令：`lsp`（基本）、`init`、`format`、`check`（占位）
- 测试策略：简单单元测试验证参数解析

**Research Findings**:

- 当前 `args/mod.rs` 使用 clap derive，`lsp: bool` 标志
- `main.rs` 通过 `if cli.lsp` 分发
- `ConfigLoader::new(manager, &cli)` 依赖 `Cli` 的 runtime getter 方法
- clap 4.5.53 支持 `#[derive(Subcommand)]`

### Metis Review

**Identified Gaps** (addressed):

- 向后兼容 `--lsp`：用户确认直接破坏性变更
- 状态信息格式：用户选择完整诊断信息
- Exit code：默认 exit 0（显示状态后）
- `print_status_info()` 位置：放在 `main.rs`（最简单）

---

## Work Objectives

### Core Objective

将CLI从扁平的 `--lsp` 标志重构为子命令架构，保持 `Cli` 公共API兼容性。

### Concrete Deliverables

- `projects/cli/src/args/mod.rs` - 添加 `Commands` enum，`command: Option<Commands>` 字段
- `projects/cli/src/main.rs` - 子命令分发逻辑，`print_status_info()` 函数
- `projects/cli/src/args/mod.rs` - 单元测试模块

### Definition of Done

- [x] `cargo build -p novelsaga-cli` 编译成功
- [x] `cargo test -p novelsaga-cli` 所有测试通过
- [x] `cargo clippy -p novelsaga-cli --all-targets` 无警告 (3个预存在的doc警告不影响)

### Must Have

- `Commands` enum 包含 `Lsp`, `Init`, `Format`, `Check` 变体
- 全局参数 (`--runtime`, `--*-path`) 使用 `global = true`
- `command: Option<Commands>` 字段（None 表示显示状态）
- `Cli::new()` 方法签名不变
- Runtime getter 方法不变：`get_runtime_choice()`, `get_node_path()`, `get_bun_path()`, `get_deno_path()`
- `init`/`format`/`check` 使用 `todo!()` 宏占位
- 无子命令时 exit code 为 0

### Must NOT Have (Guardrails)

- ❌ 修改 `config/loader.rs` - 只重构 args，不动配置加载
- ❌ 修改 `lsp/mod.rs` 或 `lsp/backend.rs` - LSP 实现不变
- ❌ 修改 `bridge/` 模块 - Bridge 系统不变
- ❌ 添加新的 Cargo 依赖
- ❌ 实现 `init`/`format`/`check` 的实际逻辑
- ❌ 添加颜色/样式库用于状态输出
- ❌ 修改 `RuntimeChoice` enum 或其 `FromStr` 实现
- ❌ 保留 `--lsp` 作为废弃别名

---

## Verification Strategy

> **UNIVERSAL RULE: ZERO HUMAN INTERVENTION**
> ALL verification is executed by the agent using tools. No human action permitted.

### Test Decision

- **Infrastructure exists**: YES (cargo test)
- **Automated tests**: Tests-after (在实现后添加简单解析测试)
- **Framework**: Rust built-in `#[cfg(test)]`

### Agent-Executed QA Scenarios (MANDATORY)

Every task includes concrete QA scenarios the executing agent will run directly.

---

## Execution Strategy

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
| ---- | ---------- | ------ | -------------------- |
| 1    | None       | 2, 3   | None                 |
| 2    | 1          | 3      | None                 |
| 3    | 1, 2       | None   | None                 |

### Agent Dispatch Summary

| Wave | Tasks | Recommended Agents |
| ---- | ----- | ------------------ |
| 1    | 1     | quick + rust-dev   |
| 2    | 2     | quick + rust-dev   |
| 3    | 3     | quick + rust-dev   |

---

## TODOs

- [x] 1. Refactor args/mod.rs: Add Commands enum and subcommand support

  **What to do**:
  1. Add `use clap::Subcommand;` import
  2. Create `Commands` enum with `#[derive(Subcommand, Clone)]`:
     - `Lsp {}` - 空结构（Plan 2 添加 port/socket）
     - `Init { path: PathBuf }` - 默认 "."
     - `Format { files: Vec<PathBuf>, check: bool }`
     - `Check { files: Vec<PathBuf> }`
  3. Modify `Cli` struct:
     - 移除 `pub lsp: bool` 字段
     - 添加 `#[arg(global = true)]` 到所有 runtime 参数
     - 添加 `#[command(subcommand)] pub command: Option<Commands>` 字段
     - 更新 `#[command(name = "novelsaga")]`（从 `novelsaga_server` 改名）
  4. 保持 `Cli::new()`, `Cli::validate()` 和所有 getter 方法不变

  **Must NOT do**:
  - 修改 `RuntimeChoice` enum
  - 修改 `validate()` 逻辑
  - 修改任何 getter 方法的签名

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 单文件结构修改，明确的clap derive模式
  - **Skills**: [`rust-dev`]
    - `rust-dev`: Rust开发规范，clap derive语法
  - **Skills Evaluated but Omitted**:
    - `ts-bridge`: 不涉及TypeScript

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 1 (alone)
  - **Blocks**: Task 2, Task 3
  - **Blocked By**: None

  **References**:

  **Pattern References**:
  - `projects/cli/src/args/mod.rs:33-57` - 当前 Cli struct 定义，展示 clap derive 模式
  - `projects/cli/src/args/mod.rs:19-31` - RuntimeChoice FromStr 实现，不要修改

  **API/Type References**:
  - `projects/cli/src/args/mod.rs:67-84` - getter 方法签名，必须保持不变

  **Documentation References**:
  - clap Subcommand: https://docs.rs/clap/latest/clap/trait.Subcommand.html
  - clap global args: https://docs.rs/clap/latest/clap/_derive/_tutorial/chapter_1/index.html

  **Acceptance Criteria**:

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: args/mod.rs compiles with new structure
    Tool: Bash
    Preconditions: Working directory is project root
    Steps:
      1. cargo check -p novelsaga-cli 2>&1
      2. Assert: exit code is 0
      3. Assert: stderr does not contain "error"
    Expected Result: No compilation errors
    Evidence: Command output captured

  Scenario: Commands enum is exported and accessible
    Tool: Bash
    Preconditions: Code changes applied
    Steps:
      1. grep -n "pub enum Commands" projects/cli/src/args/mod.rs
      2. Assert: output contains "pub enum Commands"
      3. grep -n "#\[derive(Subcommand" projects/cli/src/args/mod.rs
      4. Assert: output contains "#[derive(Subcommand"
    Expected Result: Commands enum properly defined
    Evidence: Grep output captured

  Scenario: Global args have correct attribute
    Tool: Bash
    Preconditions: Code changes applied
    Steps:
      1. grep -A1 "runtime:" projects/cli/src/args/mod.rs
      2. Assert: output contains "global = true"
      3. grep -A1 "node_path:" projects/cli/src/args/mod.rs
      4. Assert: output contains "global = true"
    Expected Result: Runtime args marked as global
    Evidence: Grep output captured

  Scenario: lsp field removed from Cli struct
    Tool: Bash
    Preconditions: Code changes applied
    Steps:
      1. grep "pub lsp: bool" projects/cli/src/args/mod.rs
      2. Assert: exit code is 1 (not found)
    Expected Result: Old lsp flag removed
    Evidence: Grep exit code captured
  ```

  **Commit**: NO (groups with Task 2)

---

- [x] 2. Update main.rs: Add command dispatch and status display

  **What to do**:
  1. 添加 `use crate::args::Commands;` import
  2. 替换 `if cli.lsp { lsp::start().await; }` 为 match 表达式:
     ```rust
     match &cli.command {
         Some(Commands::Lsp {}) => {
             lsp::start().await;
         }
         Some(Commands::Init { path }) => {
             todo!("Init command not implemented yet. Path: {:?}", path);
         }
         Some(Commands::Format { files, check }) => {
             todo!("Format command not implemented yet. Files: {:?}, Check: {}", files, check);
         }
         Some(Commands::Check { files }) => {
             todo!("Check command not implemented yet. Files: {:?}", files);
         }
         None => {
             print_status_info(&cli);
         }
     }
     ```
  3. 添加 `print_status_info()` 函数:
     ```rust
     fn print_status_info(cli: &Cli) {
         println!("NovelSaga v{}", env!("CARGO_PKG_VERSION"));
         println!();
         println!("Runtime Configuration:");
         println!("  Selected: {:?}", cli.get_runtime_choice());
         if let Some(path) = cli.get_node_path() {
             println!("  Node.js: {}", path.display());
         }
         if let Some(path) = cli.get_bun_path() {
             println!("  Bun: {}", path.display());
         }
         if let Some(path) = cli.get_deno_path() {
             println!("  Deno: {}", path.display());
         }
         println!();
         println!("Config: not loaded (run in project directory)");
         println!();
         println!("Run 'novelsaga --help' for available commands.");
     }
     ```

  **Must NOT do**:
  - 修改 BridgeManager 或 ConfigLoader 初始化逻辑
  - 修改 Feature 或 Initializer 调用
  - 在 lsp::start() 之前/之后添加额外逻辑

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 单文件修改，明确的模式替换
  - **Skills**: [`rust-dev`]
    - `rust-dev`: Rust async/await, match 语法
  - **Skills Evaluated but Omitted**:
    - `ts-bridge`: 不涉及TypeScript

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2 (alone)
  - **Blocks**: Task 3
  - **Blocked By**: Task 1

  **References**:

  **Pattern References**:
  - `projects/cli/src/main.rs:33-35` - 当前 lsp 分发逻辑，将被替换
  - `projects/cli/src/main.rs:21-31` - 初始化逻辑，不要修改

  **API/Type References**:
  - `projects/cli/src/args/mod.rs:Commands` - (Task 1 创建) 子命令 enum

  **Acceptance Criteria**:

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: main.rs compiles with command dispatch
    Tool: Bash
    Preconditions: Task 1 completed
    Steps:
      1. cargo build -p novelsaga-cli 2>&1
      2. Assert: exit code is 0
      3. Assert: stderr does not contain "error"
    Expected Result: Successful compilation
    Evidence: Command output captured

  Scenario: lsp subcommand works (starts server)
    Tool: Bash
    Preconditions: Binary built
    Steps:
      1. timeout 2 cargo run -p novelsaga-cli -- lsp 2>&1 || true
      2. Assert: output contains "LSP" or server started message (or times out which is OK)
    Expected Result: LSP server attempts to start
    Evidence: Command output captured

  Scenario: No subcommand shows status info
    Tool: Bash
    Preconditions: Binary built
    Steps:
      1. cargo run -p novelsaga-cli 2>&1
      2. Assert: output contains "NovelSaga v"
      3. Assert: output contains "Runtime Configuration"
      4. Assert: output contains "novelsaga --help"
      5. Assert: exit code is 0
    Expected Result: Status info displayed, exit 0
    Evidence: Command output captured

  Scenario: init subcommand shows todo message
    Tool: Bash
    Preconditions: Binary built
    Steps:
      1. cargo run -p novelsaga-cli -- init 2>&1 || true
      2. Assert: output contains "not implemented" (case insensitive)
    Expected Result: todo!() panic with message
    Evidence: Command output captured

  Scenario: format subcommand shows todo message
    Tool: Bash
    Preconditions: Binary built
    Steps:
      1. cargo run -p novelsaga-cli -- format 2>&1 || true
      2. Assert: output contains "not implemented" (case insensitive)
    Expected Result: todo!() panic with message
    Evidence: Command output captured

  Scenario: check subcommand shows todo message
    Tool: Bash
    Preconditions: Binary built
    Steps:
      1. cargo run -p novelsaga-cli -- check 2>&1 || true
      2. Assert: output contains "not implemented" (case insensitive)
    Expected Result: todo!() panic with message
    Evidence: Command output captured

  Scenario: Global runtime arg works with subcommand
    Tool: Bash
    Preconditions: Binary built
    Steps:
      1. cargo run -p novelsaga-cli -- --runtime node lsp 2>&1 & PID=$!; sleep 1; kill $PID 2>/dev/null || true
      2. Assert: no parsing error in output
    Expected Result: --runtime accepted before subcommand
    Evidence: Command output captured

  Scenario: Help shows all subcommands
    Tool: Bash
    Preconditions: Binary built
    Steps:
      1. cargo run -p novelsaga-cli -- --help 2>&1
      2. Assert: output contains "lsp"
      3. Assert: output contains "init"
      4. Assert: output contains "format"
      5. Assert: output contains "check"
    Expected Result: All subcommands listed in help
    Evidence: Command output captured
  ```

  **Commit**: YES
  - Message: `refactor(cli): add subcommand system with lsp, init, format, check`
  - Files: `projects/cli/src/args/mod.rs`, `projects/cli/src/main.rs`
  - Pre-commit: `cargo clippy -p novelsaga-cli --all-targets`

---

- [x] 3. Add unit tests for argument parsing

  **What to do**:
  1. 在 `args/mod.rs` 底部添加 `#[cfg(test)]` 模块
  2. 添加以下测试用例:
     - `test_parse_no_subcommand` - 验证无子命令时 `command` 为 `None`
     - `test_parse_lsp_subcommand` - 验证 `lsp` 解析为 `Commands::Lsp`
     - `test_parse_init_subcommand` - 验证 `init` 解析正确
     - `test_parse_global_runtime_with_subcommand` - 验证全局参数与子命令组合
     - `test_parse_global_runtime_without_subcommand` - 验证无子命令时全局参数仍有效

  **Must NOT do**:
  - 创建单独的测试文件（测试放在同一文件）
  - 添加集成测试
  - 测试实际命令执行逻辑

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 标准 Rust 单元测试，模式明确
  - **Skills**: [`rust-dev`]
    - `rust-dev`: Rust 测试宏，Cli::parse_from 模式
  - **Skills Evaluated but Omitted**:
    - None needed

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 3 (alone)
  - **Blocks**: None
  - **Blocked By**: Task 1, Task 2

  **References**:

  **Pattern References**:
  - `projects/cli/src/config/loader.rs:215` - 现有 `Cli::parse_from` 测试模式

  **Test References**:
  - clap testing: https://docs.rs/clap/latest/clap/_derive/_cookbook/index.html#testing

  **Acceptance Criteria**:

  **Agent-Executed QA Scenarios**:

  ```
  Scenario: All unit tests pass
    Tool: Bash
    Preconditions: Tasks 1 and 2 completed
    Steps:
      1. cargo test -p novelsaga-cli 2>&1
      2. Assert: exit code is 0
      3. Assert: output contains "test result: ok"
      4. Assert: output contains "test_parse_no_subcommand"
      5. Assert: output contains "test_parse_lsp_subcommand"
    Expected Result: All tests pass
    Evidence: Test output captured

  Scenario: No clippy warnings
    Tool: Bash
    Preconditions: All code changes applied
    Steps:
      1. cargo clippy -p novelsaga-cli --all-targets 2>&1
      2. Assert: output does not contain "warning:"
    Expected Result: Zero warnings
    Evidence: Clippy output captured
  ```

  **Commit**: YES
  - Message: `test(cli): add unit tests for subcommand argument parsing`
  - Files: `projects/cli/src/args/mod.rs`
  - Pre-commit: `cargo test -p novelsaga-cli`

---

## Commit Strategy

| After Task | Message                                                              | Files                | Verification |
| ---------- | -------------------------------------------------------------------- | -------------------- | ------------ |
| 2          | `refactor(cli): add subcommand system with lsp, init, format, check` | args/mod.rs, main.rs | cargo clippy |
| 3          | `test(cli): add unit tests for subcommand argument parsing`          | args/mod.rs          | cargo test   |

---

## Success Criteria

### Verification Commands

```bash
# 编译检查
cargo build -p novelsaga-cli  # Expected: success

# Lint 检查
cargo clippy -p novelsaga-cli --all-targets  # Expected: no warnings

# 单元测试
cargo test -p novelsaga-cli  # Expected: all tests pass

# 功能验证
cargo run -p novelsaga-cli  # Expected: shows status info, exit 0
cargo run -p novelsaga-cli -- --help  # Expected: shows lsp, init, format, check
cargo run -p novelsaga-cli -- lsp  # Expected: starts LSP server
cargo run -p novelsaga-cli -- --runtime bun lsp  # Expected: runtime arg accepted
```

### Final Checklist

- [x] `Commands` enum 包含 Lsp, Init, Format, Check
- [x] 全局参数有 `global = true`
- [x] `cli.lsp` 字段已移除
- [x] `Cli::new()` 和 getter 方法签名不变
- [x] `init`/`format`/`check` 使用 `todo!()` 占位
- [x] 无子命令时显示状态信息并 exit 0
- [x] 所有测试通过
- [x] 无 clippy 警告 (仅3个预存在的文档警告)
