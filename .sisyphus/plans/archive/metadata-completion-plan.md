
---

## Post-Completion Audit (2026-03-12)

### Metadata Storage Contract Remediation

Following the completion of this plan, an audit identified inconsistent storage paths between the CLI and LSP entrypoints (R1-R6 in the Risk Register). A subsequent remediation was performed to unify the storage contract:

- **Canonical Path**: `<workspace>/.cache/novelsaga/sled` is now the single source of truth for metadata storage.
- **Unified Resolver**: Implementation of `MetadataResolver` in `projects/cli/src/metadata/resolver.rs` ensures path consistency across all modules.
- **Legacy Migration**: Automatic migration from `.novelsaga/cache/index` to the new standard path has been verified.
- **Global Fallback Removal**: Strictly project-scoped storage policy enforced to prevent cross-project contamination.
- **Verification**: Integration tests (`tests/cli_lsp_integration.rs`) confirm that both CLI and LSP reliably read and write to the same database under identical workspace roots.
# NovelSaga Metadata System Completion Plan

## TL;DR

> **目标**: 完成 NovelSaga 元数据系统，修复 CRITICAL-1 索引问题，实现 CLI 命令，建立 CLI-to-LSP 路由机制
> 
> **交付物**:
> - 修复后的 IndexManager（解决幽灵索引问题）
> - 可用的 CLI 命令（index, list, show）
> - CLI-to-LSP 路由层（避免 sled 多进程冲突）
> - 完整的测试覆盖
> 
> **预计工作量**: Medium (3-5 天)
> **并行执行**: YES - 4 个 Wave
> **关键路径**: CRITICAL-1 修复 → CLI 命令 → CLI-to-LSP 路由

---

## Context

### 当前状态

**IndexManager** (`projects/cli/src/metadata/index.rs`):
- ✅ 已实现 sled 持久化存储
- ✅ 已实现二级索引（name, type, namespace）
- ❌ **CRITICAL-1 BUG**: `index_entity()` 在实体更新时不清理旧二级索引
- ✅ 已有 `remove_entity()` 正确清理所有索引
- ✅ 已有完整单元测试

**CLI 命令** (`projects/cli/src/commands/metadata.rs`):
- ❌ 所有处理器都是 `todo!()` 存根
- ❌ metadata 模块导出被注释掉

**LSP 后端** (`projects/cli/src/lsp/backend.rs`):
- ✅ 基本文档钩子实现（didOpen/didChange/didClose）
- ❌ 无元数据特定钩子

**JSON-RPC 基础设施**:
- ✅ `RpcClient` 已存在 (`bridge/rpc/client.rs`)
- ✅ JSON-RPC 类型已定义 (`bridge/rpc/types.rs`)
- ✅ Transport trait 抽象

### Oracle 架构建议摘要

1. **CRITICAL-1 修复**: 在 `index_entity()` 开头添加 `self.remove_entity(&entity.id)?`
2. **CLI-to-LSP 路由**: 使用 Unix Domain Socket + tower_lsp 自定义方法
3. **执行顺序**: Bug 修复 → CLI 命令 → 存储后端抽象 → 路由层
4. **测试策略**: 单元测试（文件内）+ 集成测试 + 多进程测试

### Metis 风险分析摘要

1. **数据风险**: CRITICAL-1 修复可能影响现有数据
2. **并发风险**: sled 多进程访问冲突
3. **测试缺口**: 多进程场景、错误条件覆盖
4. **隐藏依赖**: CLI-to-LSP 路由依赖稳定的存储接口

---

## Work Objectives

### Core Objective
完成 NovelSaga 元数据管理系统，使其能够通过 CLI 和 LSP 两种方式可靠地索引、查询和展示文档元数据。

### Concrete Deliverables
- [x] 修复 IndexManager 的幽灵索引问题
- [x] 实现 `novelsaga index [PATH]` 命令
- [x] 实现 `novelsaga list [--detailed]` 命令
- [x] 实现 `novelsaga show <PATH>` 命令
- [x] 实现 CLI-to-LSP 路由层（Unix Domain Socket）
- [x] 添加完整的单元测试和集成测试

### Definition of Done
- [x] `cargo test -p novelsaga-cli` 全部通过
- [x] `cargo clippy --all-targets` 无警告
- [x] CLI 命令可以独立运行（直接 sled 访问）
- [x] CLI 命令可以在 LSP 运行时路由到 LSP
- [x] 无 sled "Resource busy" 错误

### Must Have
- CRITICAL-1 修复（数据正确性）
- 三个 CLI 命令的基本实现
- CLI-to-LSP 路由机制
- 单元测试覆盖所有新代码

### Must NOT Have (Guardrails)
- **不要**修改现有元数据文件格式
- **不要**破坏现有 LSP 功能（格式化等）
- **不要**在 CLI 命令中引入阻塞操作（使用 async）
- **不要**跳过错误处理（所有 Result 必须处理）
- **不要**在测试中使用真实文件路径（使用 tempfile）

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES - 已有 `#[cfg(test)]` 模式
- **Automated tests**: Tests-after（实现后添加测试）
- **Framework**: 内置 `cargo test`
- **Test location**: 与源码同文件（遵循现有模式）

### QA Policy
每个任务必须包含 Agent-Executed QA Scenarios。

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation - 可立即开始):
├── Task 1: CRITICAL-1 修复 [quick]
└── Task 2: 重新启用 metadata 模块导出 [quick]

Wave 2 (CLI 命令实现 - 依赖 Wave 1):
├── Task 3: 实现 handle_index() [unspecified-high]
├── Task 4: 实现 handle_list() [unspecified-high]
└── Task 5: 实现 handle_show() [unspecified-high]

Wave 3 (CLI-to-LSP 路由 - 依赖 Wave 2):
├── Task 6: Unix Socket Transport 实现 [unspecified-high]
├── Task 7: LSP 自定义方法注册 [unspecified-high]
└── Task 8: CLI 路由检测和切换逻辑 [unspecified-high]

Wave 4 (测试和验证 - 依赖 Wave 2, 3):
├── Task 9: 单元测试补充 [quick]
├── Task 10: 集成测试（多进程） [unspecified-high]
└── Task 11: E2E 测试 [unspecified-high]

Wave FINAL (审查 - 独立并行):
├── Task F1: Oracle 架构审查 [oracle]
├── Task F2: Momus 质量审查 [momus]
└── Task F3: 代码质量检查 [unspecified-high]

Critical Path: Task 1 → Task 2 → Task 3-5 → Task 6-8 → Task 10-11 → F1-F3
```

### Dependency Matrix

| Task | 依赖 | 被依赖 |
|------|------|--------|
| 1 (CRITICAL-1) | - | 2, 3, 4, 5 |
| 2 (导出) | 1 | 3, 4, 5 |
| 3 (handle_index) | 1, 2 | 6, 7, 8 |
| 4 (handle_list) | 1, 2 | 6, 7, 8 |
| 5 (handle_show) | 1, 2 | 6, 7, 8 |
| 6 (Transport) | 3, 4, 5 | 7, 8, 10 |
| 7 (LSP 方法) | 3, 4, 5 | 8, 10 |
| 8 (路由逻辑) | 3, 4, 5, 6, 7 | 10 |
| 9 (单元测试) | 1, 2, 3, 4, 5 | - |
| 10 (集成测试) | 6, 7, 8 | 11 |
| 11 (E2E 测试) | 10 | - |

---

## TODOs


- [x] 1. CRITICAL-1 修复：index_entity 清理旧二级索引

  **What to do**:
  - 在 `projects/cli/src/metadata/index.rs` 的 `index_entity()` 方法开头添加旧实体清理逻辑
  - 调用 `self.get_by_id(&entity.id)?` 检查是否已存在
  - 如果存在旧实体，调用 `self.remove_entity(&entity.id)?` 清理所有旧索引
  - 然后继续原有的序列化和索引插入逻辑

  **代码变更位置**:
  ```rust
  // 在 index_entity 方法开头添加（第50行后）
  pub fn index_entity(&self, entity: &MetadataEntity) -> Result<(), sled::Error> {
      // 清理旧索引（CRITICAL-1 修复）
      if self.get_by_id(&entity.id)?.is_some() {
          self.remove_entity(&entity.id)?;
      }
      
      // 原有逻辑继续...
  ```

  **Must NOT do**:
  - 不要修改索引键格式（保持兼容性）
  - 不要删除实体数据，只清理索引
  - 不要引入新的错误类型（使用现有 sled::Error）

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: 单行修复，已有明确方案
  - **Skills**: `rust-dev`
    - `rust-dev`: 需要理解 sled API 和错误处理

  **Parallelization**:
  - **Can Run In Parallel**: NO（必须在 Wave 1 最先完成）
  - **Parallel Group**: Wave 1（顺序执行）
  - **Blocks**: Task 2, 3, 4, 5
  - **Blocked By**: None

  **References**:
  - `projects/cli/src/metadata/index.rs:50-72` - index_entity 方法实现
  - `projects/cli/src/metadata/index.rs:129-150` - remove_entity 方法（清理模式参考）
  - `projects/cli/src/metadata/index.rs:223-273` - 现有单元测试模式

  **Acceptance Criteria**:
  - [x] `index_entity` 在实体已存在时先调用 `remove_entity`
  - [x] 现有单元测试全部通过
  - [x] 新增单元测试：验证更新实体时旧索引被清理

  **QA Scenarios**:

  ```
  Scenario: 更新实体时清理旧索引
    Tool: Bash (cargo test)
    Preconditions: 代码已修改
    Steps:
      1. 运行 `cargo test -p novelsaga-cli test_index_manager_basic_crud`
      2. 添加新测试：创建实体 → 索引 → 修改 namespace → 重新索引 → 验证旧 namespace 查询不到
      3. 运行 `cargo test -p novelsaga-cli` 全部通过
    Expected Result: 所有测试通过，包括新增测试
    Evidence: .sisyphus/evidence/task-1-critical-fix.log
  ```

  **Lint**: 修改后立即运行 `cargo clippy --all-targets`

  **Commit**: YES
  - Message: `fix(cli): clear old secondary indexes on entity update`
  - Files: `projects/cli/src/metadata/index.rs`
  - Pre-commit: `cargo test -p novelsaga-cli test_index_manager`

---

- [x] 2. 重新启用 metadata 模块导出

  **What to do**:
  - 修改 `projects/cli/src/metadata/mod.rs`
  - 取消注释所有 `pub use` 语句
  - 确保导出：IndexManager, CacheManager, FileWatcher, FileChangeEvent, WriteBackWorker, WriteTask

  **代码变更**:
  ```rust
  // 取消注释第7-10行
  pub use cache::CacheManager;
  pub use index::IndexManager;
  pub use watcher::{FileChangeEvent, FileWatcher};
  pub use worker::{WriteBackWorker, WriteTask};
  ```

  **Must NOT do**:
  - 不要修改导出名称（保持向后兼容）
  - 不要添加未实现的模块导出

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `rust-dev`

  **Parallelization**:
  - **Can Run In Parallel**: NO（Wave 1 顺序执行）
  - **Blocks**: Task 3, 4, 5
  - **Blocked By**: Task 1

  **References**:
  - `projects/cli/src/metadata/mod.rs:1-10` - 当前注释掉的导出

  **Acceptance Criteria**:
  - [x] 所有 metadata 组件被正确导出
  - [x] `cargo check -p novelsaga-cli` 无错误

  **QA Scenarios**:

  ```
  Scenario: 验证导出可用
    Tool: Bash
    Steps:
      1. 运行 `cargo check -p novelsaga-cli`
      2. 在 commands/metadata.rs 中添加 `use crate::metadata::IndexManager;`
      3. 运行 `cargo check` 验证导入成功
    Expected Result: 编译成功
    Evidence: .sisyphus/evidence/task-2-exports.log
  ```

  **Lint**: `cargo check -p novelsaga-cli`

  **Commit**: YES（可与 Task 1 合并）
  - Message: `refactor(cli): re-enable metadata module exports`
  - Files: `projects/cli/src/metadata/mod.rs`



---

- [x] 3. 实现 handle_index() CLI 命令

  **What to do**:
  - 在 `projects/cli/src/commands/metadata.rs` 中实现 `handle_index()`
  - 扫描指定目录，解析元数据文件，索引到 sled

  **Must NOT do**:
  - 不要阻塞主线程（使用 async）
  - 不要硬编码路径
  - 不要忽略解析错误

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `rust-dev`, `cli-dev`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 6, 7, 8
  - **Blocked By**: Task 1, 2

  **References**:
  - `projects/cli/src/commands/metadata.rs:52-54`
  - `projects/core/src/metadata/parser.rs`
  - `projects/core/src/metadata/model.rs`

  **Acceptance Criteria**:
  - [x] 可以索引指定目录
  - [x] 正确处理 frontmatter 解析
  - [x] 显示进度信息

  **Commit**: 
  - Message: `feat(cli): implement metadata index command`
  - Files: `projects/cli/src/commands/metadata.rs`

---

- [x] 4. 实现 handle_list() CLI 命令

  **What to do**:
  - 在 `projects/cli/src/commands/metadata.rs` 中实现 `handle_list()`
  - 列出已索引的实体，支持 `--detailed` 选项

  **Must NOT do**:
  - 不要一次性加载所有实体到内存
  - 不要破坏表格格式

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `rust-dev`, `cli-dev`

  **Parallelization**:
  - **Can Run In Parallel**: YES（与 Task 3 同时）
  - **Blocks**: Task 6, 7, 8
  - **Blocked By**: Task 1, 2

  **References**:
  - `projects/cli/src/commands/metadata.rs:57-59`
  - `projects/cli/src/metadata/index.rs:103-119`

  **Acceptance Criteria**:
  - [x] 基本列表显示（ID, type, namespace）
  - [x] --detailed 显示 frontmatter 摘要

  **Commit**: 
  - Message: `feat(cli): implement metadata list command`
  - Files: `projects/cli/src/commands/metadata.rs`

---

- [x] 5. 实现 handle_show() CLI 命令

  **What to do**:
  - 在 `projects/cli/src/commands/metadata.rs` 中实现 `handle_show()`
  - 显示指定文件的完整元数据

  **Must NOT do**:
  - 不要泄露敏感信息
  - 不要在大文件上阻塞

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `rust-dev`, `cli-dev`

  **Parallelization**:
  - **Can Run In Parallel**: YES（与 Task 3, 4 同时）
  - **Blocks**: Task 6, 7, 8
  - **Blocked By**: Task 1, 2

  **References**:
  - `projects/cli/src/commands/metadata.rs:62-64`
  - `projects/cli/src/metadata/index.rs:83-93`

  **Acceptance Criteria**:
  - [x] 通过文件路径查找实体
  - [x] 显示完整元数据

  **Commit**: 
  - Message: `feat(cli): implement metadata show command`
  - Files: `projects/cli/src/commands/metadata.rs`

---

- [x] 6. Unix Socket Transport 实现

  **What to do**:
  - 创建 `projects/cli/src/bridge/transport/unix_socket.rs`
  - 实现 `Transport` trait for Unix Domain Socket

  **Must NOT do**:
  - 不要硬编码超时值
  - 不要忽略连接错误

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `rust-dev`, `cli-dev`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 8, 10
  - **Blocked By**: Task 3, 4, 5

  **References**:
  - `projects/cli/src/bridge/transport/mod.rs`
  - `projects/cli/src/bridge/transport/stdio.rs`

  **Acceptance Criteria**:
  - [x] 实现 Transport trait
  - [x] 支持连接超时

  **Commit**: 
  - Message: `feat(cli): add unix socket transport for LSP communication`
  - Files: `projects/cli/src/bridge/transport/unix_socket.rs`

---

- [x] 7. LSP 自定义方法注册

  **What to do**:
  - 在 `projects/cli/src/lsp/backend.rs` 中添加自定义方法
  - 方法：`novelsaga/index`, `novelsaga/list`, `novelsaga/show`

  **Must NOT do**:
  - 不要阻塞 LSP 主循环
  - 不要破坏 LSP 标准功能

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `rust-dev`, `lsp-dev`

  **Parallelization**:
  - **Can Run In Parallel**: YES（与 Task 6 同时）
  - **Blocks**: Task 8, 10
  - **Blocked By**: Task 3, 4, 5

  **References**:
  - `projects/cli/src/lsp/backend.rs`

  **Acceptance Criteria**:
  - [x] 三个自定义方法已注册
  - [x] 方法可以接收参数并返回结果

  **Commit**: 
  - Message: `feat(lsp): add custom metadata methods`
  - Files: `projects/cli/src/lsp/backend.rs`

---

- [x] 8. CLI 路由检测和切换逻辑

  **What to do**:
  - 修改 CLI 命令处理器，添加 LSP 检测逻辑
  - 检测方式：检查 socket 文件是否存在
  - LSP 运行时路由到 JSON-RPC，否则直接访问 sled

  **Must NOT do**:
  - 不要让检测逻辑过于复杂
  - 不要在路由决策上阻塞太久

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `rust-dev`, `cli-dev`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 10
  - **Blocked By**: Task 3, 4, 5, 6, 7

  **References**:
  - `projects/cli/src/commands/metadata.rs:43-49`
  - `projects/cli/src/bridge/rpc/client.rs`

  **Acceptance Criteria**:
  - [x] 自动检测 LSP 是否运行
  - [x] LSP 运行时路由到 JSON-RPC
  - [x] LSP 未运行时直接访问 sled

  **Commit**: 
  - Message: `feat(cli): implement CLI-to-LSP routing`
  - Files: `projects/cli/src/commands/metadata.rs`

---

- [x] 9. 单元测试补充

  **What to do**:
  - 为所有新代码添加单元测试
  - 重点：index_entity 更新场景、CLI 命令、Unix Socket Transport

  **Must NOT do**:
  - 不要测试外部依赖（使用 mock）
  - 不要跳过错误路径测试

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `rust-dev`, `testing-guide`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Blocked By**: Task 1, 2, 3, 4, 5

  **Acceptance Criteria**:
  - [x] CRITICAL-1 修复有专门测试
  - [x] 所有 public 函数有测试

  **Commit**: 
  - Message: `test(cli): add unit tests for metadata commands`

---

- [x] 10. 集成测试（多进程）

  **What to do**:
  - 创建 `tests/cli_lsp_integration.rs`
  - 测试 LSP 进程启动、CLI 路由、无 sled 冲突

  **Must NOT do**:
  - 不要在测试后留下僵尸进程
  - 不要使用固定端口/路径

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `rust-dev`, `testing-guide`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: Task 11
  - **Blocked By**: Task 6, 7, 8

  **Acceptance Criteria**:
  - [x] LSP 进程正确启动和停止
  - [x] CLI 正确路由到 LSP

  **Commit**: 
  - Message: `test(cli): add CLI-LSP integration tests`
  - Files: `tests/cli_lsp_integration.rs`

---

- [x] 11. E2E 测试

  **What to do**:
  - 使用 xtask e2e 框架或创建端到端测试脚本
  - 测试完整工作流：index → list → show

  **Must NOT do**:
  - 不要依赖外部工具

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: `testing-guide`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocked By**: Task 10

  **Acceptance Criteria**:
  - [x] 完整工作流测试通过
  - [x] 覆盖 LSP 和独立 CLI 两种模式

  **Commit**: 
  - Message: `test(cli): add E2E tests for metadata workflow`

---

## Final Verification Wave

### F1. Oracle 架构审查

**Agent**: `oracle`
**审查重点**:
- 架构决策是否符合 NovelSaga 整体设计
- CLI-to-LSP 路由方案是否合理
- 是否有遗漏的边界情况

**通过标准**: 无重大架构问题

---

### F2. Momus 质量审查

**Agent**: `momus`
**审查重点**:
- 文件引用准确性
- 任务完整性
- QA Scenario 可执行性

**通过标准**: 100% 文件引用验证

---

### F3. 代码质量检查

**检查项**:
- [x] `cargo clippy --all-targets` 无警告
- [x] `cargo test -p novelsaga-cli` 全部通过
- [x] `cargo fmt --check` 通过
- [x] 文档构建成功

---

## Commit Strategy

### Wave 1
```
fix(cli): clear old secondary indexes on entity update
refactor(cli): re-enable metadata module exports
```

### Wave 2
```
feat(cli): implement metadata index command
feat(cli): implement metadata list command
feat(cli): implement metadata show command
```

### Wave 3
```
feat(cli): add unix socket transport for LSP communication
feat(lsp): add custom metadata methods
feat(cli): implement CLI-to-LSP routing
```

### Wave 4
```
test(cli): add comprehensive tests for metadata system
```

---

## Success Criteria

### Verification Commands
```bash
cargo build -p novelsaga-cli
cargo test -p novelsaga-cli
cargo clippy --all-targets -p novelsaga-cli
cargo fmt --check -p novelsaga-cli
```

### Final Checklist
- [x] CRITICAL-1 修复已合并
- [x] CLI index 命令可用
- [x] CLI list 命令可用
- [x] CLI show 命令可用
- [x] CLI-to-LSP 路由工作正常
- [x] 无 sled "Resource busy" 错误
- [x] 所有测试通过
- [x] clippy 无警告
- [x] Oracle 审查通过
- [x] Momus 审查通过

---

## Appendix: 参考资料

### 关键文件位置
```
projects/cli/src/metadata/
├── index.rs          # IndexManager
├── mod.rs            # 模块导出
├── cache.rs          # CacheManager
├── watcher.rs        # FileWatcher
└── worker.rs         # WriteBackWorker

projects/cli/src/commands/metadata.rs  # CLI 命令处理器
projects/cli/src/lsp/backend.rs        # LSP 后端
projects/cli/src/bridge/rpc/client.rs  # RpcClient
projects/cli/src/bridge/transport/     # Transport trait

projects/core/src/metadata/
├── model.rs          # MetadataEntity
└── parser.rs         # 元数据解析工具
```

### 遇到问题时的 Subagent
| 问题类型 | 推荐 Subagent |
|----------|---------------|
| 架构决策 | `oracle` |
| 代码质量 | `momus` |
| 实现细节 | `Sisyphus-Junior` |
| 测试设计 | `metis` |

---

*Plan generated by Prometheus*
*Last updated: 2026-02-27*