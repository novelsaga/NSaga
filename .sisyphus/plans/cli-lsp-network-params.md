# CLI LSP Network Parameters (Plan 2)

## TL;DR

> **Quick Summary**: 为 `lsp` 子命令添加 `--port` 和 `--socket` 参数，支持 TCP 和 Unix Socket 通信模式（除 stdio 外）。
>
> **Deliverables**:
>
> - `args/mod.rs`: Lsp 变体添加 port/socket 字段
> - `lsp/mod.rs`: 扩展 `start()` 函数支持多种传输模式
>
> **Estimated Effort**: Short (1-2 hours)
> **Parallel Execution**: NO - sequential
> **Critical Path**: Task 1 (args) → Task 2 (lsp transport)
> **Depends On**: Plan 1 (cli-subcommand-base) 必须先完成

---

## Context

### Original Request

用户希望 LSP 子命令支持网络参数：

- `--port <u16>` - TCP 模式，通过指定端口通信
- `--socket <path>` - Unix Socket 模式

### Prerequisites

- **Plan 1 必须完成**: `Commands::Lsp {}` 变体已存在
- `main.rs` 已有命令分发逻辑

### Interview Summary

**Key Discussions**:

- 参数先添加结构，实现可以抛出 `todo!()` 或实际实现
- 这是 LSP 子命令专属参数，不是全局参数

---

## Work Objectives

### Core Objective

扩展 LSP 子命令，支持 TCP 和 Unix Socket 传输模式。

### Concrete Deliverables

- `projects/cli/src/args/mod.rs` - `Lsp` 变体添加 `port` 和 `socket` 字段
- `projects/cli/src/lsp/mod.rs` - `start()` 函数接受传输配置参数
- `projects/cli/src/main.rs` - 传递参数给 `lsp::start()`

### Definition of Done

- [ ] `cargo build -p novelsaga-cli` 编译成功
- [ ] `novelsaga lsp --port 9999` 解析成功（可以 todo 实现）
- [ ] `novelsaga lsp --socket /tmp/ns.sock` 解析成功
- [ ] `novelsaga lsp` (无参数) 仍使用 stdio 模式

### Must Have

- `--port` 和 `--socket` 互斥（不能同时指定）
- 默认行为（无参数）保持 stdio 模式不变
- 参数验证：port 范围、socket 路径有效性

### Must NOT Have (Guardrails)

- ❌ 修改 LSP backend 逻辑 (`lsp/backend.rs`)
- ❌ 添加新的网络依赖（tower-lsp 已支持 TCP）
- ❌ 实现完整的 TCP/Socket 服务器（可以 todo）

---

## Verification Strategy

### Test Decision

- **Automated tests**: Tests-after (参数解析测试)
- **Framework**: Rust built-in `#[cfg(test)]`

### Agent-Executed QA Scenarios (MANDATORY)

见各 TODO 任务。

---

## Execution Strategy

### Dependency Matrix

| Task | Depends On  | Blocks |
| ---- | ----------- | ------ |
| 1    | Plan 1 完成 | 2      |
| 2    | 1           | None   |

---

## TODOs

- [ ] 1. Add port and socket parameters to Lsp subcommand

  **What to do**:
  1. 修改 `Commands::Lsp` 变体添加字段:
     ```rust
     Lsp {
         /// TCP port for network mode (mutually exclusive with --socket)
         #[arg(long, conflicts_with = "socket")]
         port: Option<u16>,

         /// Unix socket path for socket mode (mutually exclusive with --port)
         #[arg(long, conflicts_with = "port")]
         socket: Option<PathBuf>,
     },
     ```
  2. 更新 `main.rs` 中的 match 分支提取参数:
     ```rust
     Some(Commands::Lsp { port, socket }) => {
         lsp::start(port, socket).await;
     }
     ```

  **Must NOT do**:
  - 修改其他子命令
  - 添加为全局参数

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: [`rust-dev`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 2
  - **Blocked By**: Plan 1

  **References**:

  **Pattern References**:
  - `projects/cli/src/args/mod.rs:Commands::Lsp` - (Plan 1 创建) 当前 Lsp 变体
  - clap conflicts_with: https://docs.rs/clap/latest/clap/_derive/index.html#arg-attributes

  **Acceptance Criteria**:

  ```
  Scenario: Port parameter parses correctly
    Tool: Bash
    Steps:
      1. cargo run -p novelsaga-cli -- lsp --port 9999 --help 2>&1 || true
      2. cargo run -p novelsaga-cli -- lsp --help 2>&1
      3. Assert: help output contains "--port"
    Expected Result: Port parameter recognized
    Evidence: Help output captured

  Scenario: Socket parameter parses correctly
    Tool: Bash
    Steps:
      1. cargo run -p novelsaga-cli -- lsp --help 2>&1
      2. Assert: help output contains "--socket"
    Expected Result: Socket parameter recognized
    Evidence: Help output captured

  Scenario: Port and socket are mutually exclusive
    Tool: Bash
    Steps:
      1. cargo run -p novelsaga-cli -- lsp --port 9999 --socket /tmp/x.sock 2>&1 || true
      2. Assert: output contains "cannot be used with" or "conflict"
    Expected Result: Conflict error shown
    Evidence: Error output captured
  ```

  **Commit**: NO (groups with Task 2)

---

- [ ] 2. Update lsp::start() to accept transport configuration

  **What to do**:
  1. 修改 `lsp/mod.rs` 中的 `start()` 函数签名:
     ```rust
     pub async fn start(port: Option<u16>, socket: Option<PathBuf>) {
         match (port, socket) {
             (Some(p), None) => {
                 todo!("TCP mode on port {} not implemented yet", p);
             }
             (None, Some(s)) => {
                 todo!("Unix socket mode at {:?} not implemented yet", s);
             }
             (None, None) => {
                 // 现有 stdio 逻辑
                 let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
                 // ... 保持不变
             }
             (Some(_), Some(_)) => unreachable!("clap prevents this"),
         }
     }
     ```
  2. 保持现有 stdio 逻辑不变

  **Must NOT do**:
  - 修改 `backend.rs`
  - 实现完整的 TCP/Socket 服务器

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: [`rust-dev`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 1

  **References**:

  **Pattern References**:
  - `projects/cli/src/lsp/mod.rs:start()` - 当前 stdio 实现

  **Acceptance Criteria**:

  ```
  Scenario: Default stdio mode still works
    Tool: Bash
    Steps:
      1. timeout 2 cargo run -p novelsaga-cli -- lsp 2>&1 || true
      2. Assert: no "not implemented" in output (unless using --port/--socket)
    Expected Result: Stdio mode unchanged
    Evidence: Output captured

  Scenario: Port mode shows todo message
    Tool: Bash
    Steps:
      1. cargo run -p novelsaga-cli -- lsp --port 9999 2>&1 || true
      2. Assert: output contains "not implemented" or "TCP"
    Expected Result: Todo placeholder works
    Evidence: Output captured

  Scenario: Socket mode shows todo message
    Tool: Bash
    Steps:
      1. cargo run -p novelsaga-cli -- lsp --socket /tmp/test.sock 2>&1 || true
      2. Assert: output contains "not implemented" or "socket"
    Expected Result: Todo placeholder works
    Evidence: Output captured
  ```

  **Commit**: YES
  - Message: `feat(cli): add --port and --socket parameters to lsp subcommand`
  - Files: `projects/cli/src/args/mod.rs`, `projects/cli/src/main.rs`, `projects/cli/src/lsp/mod.rs`
  - Pre-commit: `cargo clippy -p novelsaga-cli --all-targets`

---

## Success Criteria

### Verification Commands

```bash
cargo build -p novelsaga-cli
cargo run -p novelsaga-cli -- lsp --help  # Shows --port and --socket
cargo run -p novelsaga-cli -- lsp  # Stdio mode (unchanged)
cargo run -p novelsaga-cli -- lsp --port 9999  # Todo message
cargo run -p novelsaga-cli -- lsp --socket /tmp/x.sock  # Todo message
```

### Final Checklist

- [ ] `--port` 和 `--socket` 参数存在
- [ ] 两参数互斥
- [ ] 默认 stdio 模式不变
- [ ] TCP/Socket 模式显示 todo 信息
