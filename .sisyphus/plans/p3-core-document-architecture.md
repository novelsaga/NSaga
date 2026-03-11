# P3 Phase 1: Core Document Architecture

## TL;DR

> **目标**: 建立 NovelSaga P3 LSP 集成的核心文档架构，支持 metadata 和 article 的统一建模
>
> **交付物**:
>
> - `MarkdownParts` 共享解析层
> - `ArticleDocument` 结构化实体（替代现有简单 `Article`）
> - `MetadataEntity::from_parts()` 构造方法
> - `WorkspaceDocument` 统一入口 enum
> - LSP `DocumentState`（支持解析错误保留、workspace_root、并发约束）
> - LSP `didChangeWatchedFiles` 完整支持（含 path->id 反向索引）
> - 集成测试（async-lsp-client）
>
> **预计工作量**: Medium (1.5-2 天)
> **并行执行**: YES - 3 个 Wave
> **关键路径**: Wave 1 (Core) → Wave 2 (LSP) → Wave 3 (Tests)

---

## Context

### 背景

P3 LSP 集成需要处理两类文档：

- **Metadata**: `**/metadata/**/*.md`，有 `id`, `type_`, `namespace` 等字段
- **Article**: 其他 `.md` 文件，有 `chapter_number`, `title`, `order` 等字段

两者都是 markdown 文件，都有 frontmatter 和 body，解析流程相同，但结构化字段不同。

### 当前状态

**Core 层** (`projects/core/src/`):

- ✅ `MetadataEntity`: 完整结构，有 TS 导出
- ⚠️ `Article`: 只有 `content: String`，过于简单
- ❌ 无共享解析层
- ❌ 无统一文档入口

**CLI 层** (`projects/cli/src/`):

- ✅ `watcher.rs`: FileWatcher 组件（P2 遗留，**本次弃用**）
- ✅ LSP backend: 基础 didOpen/didChange/didClose
- ❌ 无 didChangeWatchedFiles 支持
- ❌ DocumentStore 是 `HashMap<Url, String>`，无结构化解析

### 关键决策（已确认）

| 决策项       | 决定                                |
| ------------ | ----------------------------------- |
| FileWatcher  | ❌ 不使用，只使用 LSP watched-files |
| CLI 模式     | ✅ 显式刷新，无后台监控             |
| Article 迁移 | ✅ 直接重构（原结构简单，代码量少） |
| 文件监控     | ✅ LSP didChangeWatchedFiles 足够   |

---

## Work Objectives

### Core Objective

建立支持 metadata 和 article 统一建模的文档架构，为 P3 后续 Hover、Completion 功能奠定基础。

### Concrete Deliverables

- [x] `projects/core/src/document/mod.rs` 创建（`MarkdownParts`, `DocumentKind`, `classify_path`）
- [x] `ArticleDocument` 结构定义（含 `from_parts` 构造模式）
- [x] `MetadataEntity::from_parts()` 实现
- [x] `WorkspaceDocument` enum 定义
- [x] LSP `DocumentState` 升级（含 workspace_root、并发约束）
- [x] LSP `didChangeWatchedFiles` 完整支持（含 path->id 反向索引）
- [x] IndexManager path->id 反向索引实现
- [x] FileWatcher 弃用
- [x] 集成测试（async-lsp-client，位于 cli crate）

### Definition of Done

- [x] `cargo test -p novelsaga-core` 全部通过
- [x] `cargo test -p novelsaga-cli` 全部通过
- [x] `cargo clippy --all-targets` 无警告
- [x] LSP 测试能成功打开/修改/关闭文档
- [x] didChangeWatchedFiles 能正确处理 create/change/delete/rename
- [x] workspace_root 正确保存（含 fallback 处理）
- [x] path->id 反向索引工作正常

### Must NOT Have (Guardrails)

- **不要**使用 FileWatcher（watcher.rs）
- **不要**添加后台监控进程
- **不要**修改 MetadataEntity 的 TS 导出结构
- **不要**在 core 层引入 LSP 类型依赖
- **不要**过早优化（如 rope 结构、增量同步）

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Core - 可立即开始):
├── Task 1: 创建 document 模块和 MarkdownParts [quick]
├── Task 2: 重构 Article 为 ArticleDocument [quick]
├── Task 3: 添加 MetadataEntity::from_parts() [quick]
└── Task 4: 弃用 FileWatcher [quick]

Wave 2 (LSP 集成 - 依赖 Wave 1):
├── Task 5: 创建 WorkspaceDocument enum [quick]
├── Task 6: 升级 LSP DocumentStore 为 DocumentState [unspecified-high]
└── Task 7: 添加 didChangeWatchedFiles 支持（含反向索引） [unspecified-high]

Wave 3 (测试 - 依赖 Wave 2):
├── Task 8: 单元测试补充 [quick]
└── Task 9: LSP 集成测试 [unspecified-high]

Wave FINAL (审查 - 独立并行):
├── Task F1: Plan 合规审计 [oracle]
└── Task F2: 代码质量检查 [unspecified-high]

Critical Path: Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6 → Task 7 → Task 9 → F1-F2
```

### Dependency Matrix

| Task                                 | 依赖 | 被依赖 |
| ------------------------------------ | ---- | ------ |
| 1 (document 模块)                    | -    | 2, 3   |
| 2 (ArticleDocument)                  | 1    | 5      |
| 3 (MetadataEntity from_parts)        | 1    | 5      |
| 4 (弃用 FileWatcher)                 | -    | -      |
| 5 (WorkspaceDocument)                | 2, 3 | 6      |
| 6 (DocumentState + workspace_root)   | 5    | 7, 8   |
| 7 (didChangeWatchedFiles + 反向索引) | 6    | 9      |
| 8 (单元测试)                         | 6    | -      |
| 9 (集成测试)                         | 7    | F1     |

---

## TODOs

- [x] 1. 创建 core document 模块和 MarkdownParts

  **What to do**:
  - 创建 `projects/core/src/document/mod.rs`
  - 定义 `MarkdownParts` 结构体：`frontmatter: Value`, `body: String`, `has_frontmatter: bool`
  - 定义 `DocumentKind` enum：`Metadata`, `Article`
  - 实现 `classify_path()` 函数：根据路径判断文档类型
    - `**/metadata/**/*.md` → `DocumentKind::Metadata`
    - 其他 `.md` → `DocumentKind::Article`
  - 添加模块导出到 `projects/core/src/lib.rs`

  **Must NOT do**:
  - 不要在 core 层引入 LSP 类型（Url 等）
  - 不要添加 async 代码到 core
  - 不要修改现有 metadata/article 模块的导出

  **Acceptance Criteria**:
  - [x] `MarkdownParts` 可正确分离 frontmatter 和 body
  - [x] `classify_path()` 正确识别 metadata/article 路径
  - [x] `cargo test -p novelsaga-core` 新增测试通过

---

- [x] 2. 重构 Article 为 ArticleDocument

  **What to do**:
  - 修改 `projects/core/src/article/mod.rs`
  - 定义新的 `ArticleDocument` 结构体：
    ```rust
    pub struct ArticleDocument {
      pub frontmatter: Value,
      pub body: String,
      pub title: Option<String>,
      pub chapter_number: Option<u32>,
      pub order: Option<i32>,
      pub word_count: u32,
    }
    ```
  - 实现 `from_parts(parts: MarkdownParts) -> Self`
  - 实现 `new(frontmatter: Value, body: impl Into<String>) -> Self`（便利构造器）
  - 添加 `impl From<MarkdownParts> for ArticleDocument`

  **Must NOT do**:
  - 不要删除旧 `Article` 结构（保持向后兼容）
  - 不要添加 serde TS 导出（除非需要）
  - 不要引入 async 代码

  **Acceptance Criteria**:
  - [x] `ArticleDocument` 可正确从 `MarkdownParts` 构造
  - [x] `title`, `chapter_number`, `order` 正确从 frontmatter 提取
  - [x] `word_count` 正确计算（body.split_whitespace().count()）
  - [x] 旧 `Article` 代码仍能编译

---

- [x] 3. 添加 MetadataEntity::from_parts()

  **What to do**:
  - 修改 `projects/core/src/metadata/model.rs`
  - 定义 `MetadataEntityParts` 结构体：
    ```rust
    pub struct MetadataEntityParts {
      pub id: String,
      pub type_: String,
      pub namespace: String,
      pub frontmatter: Value,
      pub body: String,
    }
    ```
  - 实现 `impl MetadataEntity { pub fn from_parts(parts: MetadataEntityParts) -> Self }`
  - 重构现有 `new()` 方法，内部调用 `from_parts()`
  - 添加 `impl TryFrom<(MarkdownParts, &Path, &Path)> for MetadataEntity`
    - 从 MarkdownParts + 文件路径 + workspace_root 构造
    - 使用 `resolve_type()` 和 `generate_namespace()`

  **Must NOT do**:
  - 不要修改 `MetadataEntity` 的字段结构（保持 TS 导出兼容）
  - 不要删除现有 `new()` 方法
  - 不要修改现有测试

  **Acceptance Criteria**:
  - [x] `from_parts()` 可正确构造 MetadataEntity
  - [x] `new()` 内部调用 `from_parts()`
  - [x] `TryFrom` 实现正确处理路径和 frontmatter
  - [x] 所有现有测试仍通过

---

- [x] 4. 弃用 FileWatcher (watcher.rs)

  **What to do**:
  - 修改 `projects/cli/src/metadata/mod.rs`
  - 注释掉 `pub mod watcher;` 和 `pub use watcher::{...}`
  - 添加注释说明：FileWatcher 在 P3 中不使用，保留代码但禁用导出
  - 将 `watcher.rs` 标记为 deprecated（添加文档注释）

  **Must NOT do**:
  - 不要删除 `watcher.rs` 文件（保留以备后用）
  - 不要修改 watcher.rs 内部代码

  **Acceptance Criteria**:
  - [x] `watcher` 模块不再被导出
  - [x] `cargo check -p novelsaga-cli` 无警告
  - [x] 其他代码不依赖 FileWatcher

---

- [x] 5. 创建 WorkspaceDocument enum

  **What to do**:
  - 创建/修改 `projects/core/src/document/mod.rs`
  - 定义 `WorkspaceDocument` enum：
    ```rust
    pub enum WorkspaceDocument {
      Metadata(MetadataEntity),
      Article(ArticleDocument),
    }
    ```
  - 实现方法：
    - `kind() -> DocumentKind`
    - `as_metadata() -> Option<&MetadataEntity>`
    - `as_article() -> Option<&ArticleDocument>`

  **Must NOT do**:
  - 不要添加 LSP 类型依赖到 core
  - 不要添加 async 代码

  **Acceptance Criteria**:
  - [x] `WorkspaceDocument` 正确定义和实现
  - [x] match 分支能正确处理两种类型
  - [x] 单元测试覆盖两种变体

---

- [x] 6. 升级 LSP DocumentStore 为 DocumentState（含 workspace_root 和并发约束）

  **What to do**:
  - 修改 `projects/cli/src/lsp/backend.rs`
  - **添加 workspace_root 保存**：
    - 在 `initialize` 中保存 `root_uri` 转换后的 `workspace_root: PathBuf`
    - **Fallback 策略**：`root_uri` → `workspaceFolders[0]` → 从首个文件推导 → `None`
    - 统一处理 `file://` URI → `PathBuf` 转换逻辑
  - 定义 `DocumentState` 结构体：
    ```rust
    pub struct DocumentState {
      pub version: i32,
      pub text: Arc<str>,
      pub kind: DocumentKind,
      pub parsed: Result<WorkspaceDocument, String>,
      pub disk_changed: bool,
    }
    ```
  - 修改 `DocumentStore` 类型别名
  - **并发约束 - 锁外解析**：
    1. 在锁内复制 `text` 和 `version`
    2. 释放锁
    3. 在锁外执行解析（可能耗时）
    4. 重新获取锁，回写 `parsed` 结果
  - 修改 `did_open`：构造 `DocumentState` 并**同步解析**（不做 debounce）
  - 修改 `did_change`：更新 `text` 和 `version`，**同步重解析**（不做 debounce）
  - 修改 `did_close`：从 store 移除

  **Must NOT do**:
  - 不要实现 debounce（P3 Phase 1 不做）
  - 不要修改其他 LSP 方法
  - 不要破坏现有功能（formatting 等）

  **Acceptance Criteria**:
  - [x] `workspace_root` 正确保存（含 fallback）
  - [x] URI → Path 转换逻辑正确
  - [x] **并发约束实现**：锁外解析，锁内回写
  - [x] `did_open`/`did_change` 同步解析
  - [x] 解析错误时保留在 store

---

- [x] 7. 添加 LSP didChangeWatchedFiles 支持（含 delete/rename 处理）

  **What to do**:
  - 在 LSP backend 中注册 `workspace/didChangeWatchedFiles` capability
  - 实现 `did_change_watched_files` 处理函数
  - **添加 `canonical_path -> id` 独立反向索引**：
    - 在 `IndexManager` 中维护独立的 `path_to_id: HashMap<PathBuf, String>`
    - **关键约束**：`remove_entity()` 和 `rebuild_index()` 必须同步维护反向索引
    - 反向索引**仅内存存储**，LSP 重启时重建
  - **watched-files 注册方式**：
    - 在 LSP `initialize` 中尝试动态注册 `workspace/didChangeWatchedFiles`
    - 依赖客户端 capability，不支持则降级
  - 监听模式：`**/metadata/**/*.md`, `**/novelsaga.config.*`, `**/.novelsaga.*`
  - 收到变更通知时（**由测试客户端主动发送**）：
    - **Create/Change**: 解析文件，更新 IndexManager
    - **Delete**: 通过反向索引查找 id，调用 `remove_entity()` 清理
    - **Rename**: 视为 Delete + Create
  - **打开文档特殊处理**：
    - 已打开文档：**不替换** `text`，只标记 `disk_changed = true`
    - 未打开文档：直接更新索引

  **Must NOT do**:
  - 不要监听所有文件
  - 不要立即重新加载（使用 worker 异步处理）
  - **打开文档不要被 watched-files 覆盖**

  **Acceptance Criteria**:
  - [x] LSP 动态注册 watched files capability
  - [x] IndexManager 添加 `canonical_path -> id` 反向索引
  - [x] **反向索引维护**：`remove/rebuild` 同步清理/重建
  - [x] Create/Change/Delete/Rename 正确处理
  - [x] 打开文档只标记 `disk_changed`

---

- [x] 8. 单元测试补充

  **What to do**:
  - 为 Task 1-5 的所有新代码添加单元测试
  - 重点测试：MarkdownParts 解析、classify_path、ArticleDocument from_parts、MetadataEntity TryFrom、WorkspaceDocument match
  - 遵循现有测试模式：`#[cfg(test)]` 模块在同文件

  **Must NOT do**:
  - 不要测试外部依赖（使用 mock）
  - 不要重复测试现有代码

  **Acceptance Criteria**:
  - [x] 新增代码行覆盖率 >80%
  - [x] 所有边界情况有测试
  - [x] `cargo test -p novelsaga-core` 通过

---

- [x] 9. LSP 集成测试

  **What to do**:
  - 创建 `projects/cli/tests/lsp_document_integration.rs`
  - 在 `projects/cli/Cargo.toml` 的 `[dev-dependencies]` 中添加 `async-lsp-client = "0.2.3"`
  - 使用 `async-lsp-client` 测试：LSP 启动、initialize、didOpen/didChange/didClose、metadata/article 解析、错误处理
  - 使用 `tempfile` 创建隔离测试环境

  **Must NOT do**:
  - 不要留下僵尸进程
  - 不要使用固定端口/路径
  - 不要依赖外部工具

  **Acceptance Criteria**:
  - [x] `projects/cli/Cargo.toml` 添加 `async-lsp-client` dev-dependency
  - [x] 集成测试可独立运行
  - [x] 覆盖所有 LSP 文档生命周期
  - [x] 测试后无残留进程/文件

---

## Final Verification Wave

### F1. Plan 合规审计

**Agent**: `oracle`

**审查重点**:

- 架构是否符合分层设计（core 层无 LSP 依赖）
- from_parts 模式是否正确应用
- DocumentState 错误处理是否合理
- FileWatcher 是否正确弃用

**通过标准**: 无重大架构问题

---

### F2. 代码质量检查

**检查项**:

- [x] `cargo clippy --all-targets -p novelsaga-core` 无警告
- [x] `cargo clippy --all-targets -p novelsaga-cli` 无警告
- [x] `cargo test -p novelsaga-core` 全部通过
- [x] `cargo test -p novelsaga-cli` 全部通过
- [x] `cargo fmt --check` 通过
- [x] 新增代码有适当文档注释

---

## Commit Strategy

### Wave 1 (Core)

```
feat(core): add shared document parsing layer
feat(core): refactor Article to structured ArticleDocument
refactor(core): add from_parts constructor to MetadataEntity
refactor(cli): deprecate FileWatcher
```

### Wave 2 (LSP)

```
feat(core): add WorkspaceDocument enum for unified handling
feat(lsp): upgrade DocumentStore to DocumentState with workspace_root
feat(lsp): add didChangeWatchedFiles support with path-to-id index
```

### Wave 3 (Tests)

```
test(core): add unit tests for document architecture
test(cli): add LSP document integration tests
```

---

## Implementation Guidelines

### Oracle's Implementation Notes

Task 7 (`path_to_id` 反向索引) 实现时必须钉死的小点：

1. **反向索引维护原子性**: upsert/index、remove、rebuild、rename(create+delete) 都必须同步维护反向索引
2. **canonical_path 来源和失败策略**: 只有成功 canonicalize 的路径才进反向索引；失败时统一退回原始 path
3. **两条流区分**: `didOpen/didChange` 是**同步解析并更新 DocumentState**；`didChangeWatchedFiles` 是**异步索引刷新**，不要混用
4. **动态注册时机**: 若 `tower-lsp`/客户端交互受限，允许改到 `initialized()`，不要当成硬约束
5. **workspace_root 推导限定**: 只在收到第一个 `file://` 文档且 `root_uri/workspaceFolders` 都缺失时启用，避免多根工作区误判

### Code Style Guidelines

**函数式优先，必要时 OOP**:
- 优先使用纯函数、不可变数据结构
- 使用 `from_parts()` 等构造模式分离解析和组装
- 仅在需要封装状态或实现 trait 时使用 struct/impl
- 避免过度抽象，保持代码扁平

### Pre-commit Requirements (from flake.nix)

**所有 commit 前必须通过以下检查**:

```bash
# Rust 检查
cargo clippy --all-targets --all-features --workspace -- -D warnings
cargo fmt --check -p novelsaga-core -p novelsaga-cli

# JS/TS 检查
pnpm exec eslint .

# 通用格式化
treefmt
```

**禁止跳过**: 项目配置了 git hooks，上述检查会在 commit 时自动运行，**失败则无法 commit**。

### When to Consult Specialists

| 问题类型 | 咨询专家 | 查询资源 |
|----------|----------|----------|
| 架构设计、分层边界 | @oracle | Rust API Guidelines, NovelSaga AGENTS.md |
| LSP 协议、tower-lsp 使用 | @oracle | tower-lsp docs, LSP spec |
| 复杂类型系统、生命周期 | @oracle | Rustnomicon, Rust API Guidelines |
| 并发、异步、锁策略 | @oracle | Tokio docs, Rust并发模式 |
| 非传统问题、创新方案 | @artistry | 无（创造性解决） |
| 外部库使用、最佳实践 | @librarian | docs.rs, GitHub examples |
| 代码结构、文件组织 | @explore | 现有 codebase 模式 |

### Useful Resources

| 资源 | 用途 |
|------|------|
| https://docs.rs/tower-lsp/latest/tower_lsp/ | LSP 服务端实现 |
| https://microsoft.github.io/language-server-protocol/specifications/specification-current/ | LSP 协议规范 |
| https://rust-lang.github.io/api-guidelines/ | Rust API 设计指南 |
| https://doc.rust-lang.org/nomicon/ | Rust 高级主题 |
| https://docs.rs/tokio/latest/tokio/ | 异步运行时 |
| https://docs.rs/lsp-types/latest/lsp_types/ | LSP 类型定义 |

## Success Criteria

### Verification Commands

```bash
cargo build -p novelsaga-core -p novelsaga-cli
cargo test -p novelsaga-core
cargo test -p novelsaga-cli
cargo clippy --all-targets -p novelsaga-core -p novelsaga-cli
cargo fmt --check -p novelsaga-core -p novelsaga-cli
```

### Final Checklist

- [x] MarkdownParts 解析正确
- [x] ArticleDocument 结构化字段完整
- [x] MetadataEntity from_parts 可用
- [x] WorkspaceDocument 统一入口工作
- [x] DocumentState 支持解析错误保留（含 workspace_root）
- [x] didChangeWatchedFiles 完整支持（create/change/delete/rename）
- [x] path->id 反向索引工作正常
- [x] FileWatcher 已弃用
- [x] 单元测试覆盖率达标
- [x] LSP 集成测试通过（位于 cli crate）
- [x] 无 clippy 警告

---

_Plan generated by Prometheus_
_Last updated: 2026-03-11 (Rewrite - Clean Version)_
