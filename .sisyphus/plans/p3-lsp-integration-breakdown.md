# Plan: P3 - LSP 服务器集成 - 详细任务拆分

## TL;DR

将 P3 LSP 服务器集成任务拆分为细粒度的子任务，每个任务对应一个文件或功能模块的创建/修改。

## Context

- **父任务**: `.sisyphus/plans/cli-next-steps.md` - P3 LSP 服务器集成
- **预计总工作量**: 5-7 天
- **拆分原则**: 一个功能一个任务，按 LSP 协议的标准功能划分
- **可并行性**: 标注哪些任务可以并行执行

## Background

**现有架构**:

- LSP backend 骨架已存在: `projects/cli/src/lsp/backend.rs`
- 使用 `tower-lsp` crate
- 支持异步处理 (`tokio`)

**需要实现的 LSP 功能**:

1. **hover** (悬停提示) - 显示元数据信息
2. **completion** (自动补全) - 场景/章节/角色名补全
3. **textDocument/didOpen** - 文档打开通知
4. **textDocument/didChange** - 文档变更通知
5. **textDocument/didSave** - 文档保存通知
6. **textDocument/didClose** - 文档关闭通知

## TODO List

### Phase 1: 文档生命周期管理

- [ ] **Task 1.1: 定义 Document 结构体**
  - **文件**: `projects/cli/src/lsp/document.rs` (新建)
  - **内容**:
    - `Document` 结构体 (uri, content, version, metadata_id)
    - 文档内容缓存
    - 版本号跟踪
    - 关联的元数据 ID (可选)
  - **依赖**: 无
  - **可并行**: ✅ 是 (独立任务)
  - **验证**: `cargo build -p novelsaga-cli` 通过

- [ ] **Task 1.2: 定义 DocumentStore**
  - **文件**: `projects/cli/src/lsp/document.rs` (追加)
  - **内容**:
    - `DocumentStore` 结构体
    - 字段: `documents: Arc<RwLock<BTreeMap<Url, Document>>>`
    - 方法: `insert`, `get`, `update`, `remove`
    - 线程安全的文档集合管理
  - **依赖**: Task 1.1 完成
  - **可并行**: ❌ 否 (依赖 Document)
  - **验证**: `cargo test -p novelsaga-cli test_document_store`

- [ ] **Task 1.3: 实现 textDocument/didOpen 处理**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 实现 `LanguageServer::did_open` 方法
    - 解析文档 frontmatter (gray_matter)
    - 提取元数据 ID
    - 将文档添加到 DocumentStore
  - **依赖**: Task 1.2 完成
  - **可并行**: ❌ 否 (依赖 DocumentStore)
  - **验证**: 集成测试 - 打开文档后检查 store

- [ ] **Task 1.4: 实现 textDocument/didChange 处理**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 实现 `LanguageServer::did_change` 方法
    - 应用增量或全量更改
    - 更新文档版本号
    - 更新 DocumentStore
  - **依赖**: Task 1.2 完成
  - **可并行**: ✅ 是 (与 Task 1.3, 1.5, 1.6 并行)
  - **验证**: 集成测试 - 修改文档后检查内容

- [ ] **Task 1.5: 实现 textDocument/didSave 处理**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 实现 `LanguageServer::did_save` 方法
    - 触发元数据同步 (如果文档关联了元数据)
    - 记录保存事件日志
  - **依赖**: Task 1.2 完成
  - **可并行**: ✅ 是 (与 Task 1.3, 1.4, 1.6 并行)
  - **验证**: 集成测试 - 保存文档后检查日志

- [ ] **Task 1.6: 实现 textDocument/didClose 处理**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 实现 `LanguageServer::did_close` 方法
    - 从 DocumentStore 移除文档
    - 清理相关资源
  - **依赖**: Task 1.2 完成
  - **可并行**: ✅ 是 (与 Task 1.3-1.5 并行)
  - **验证**: 集成测试 - 关闭文档后检查 store 为空

### Phase 2: Hover 功能实现 (悬停提示)

- [ ] **Task 2.1: 定义 HoverProvider trait**
  - **文件**: `projects/cli/src/lsp/hover.rs` (新建)
  - **内容**:
    - `HoverProvider` trait
    - 方法: `provide_hover(&self, document: &Document, position: Position) -> Option<Hover>`
  - **依赖**: Task 1.1 完成
  - **可并行**: ✅ 是 (与 Phase 1 并行)
  - **验证**: `cargo build -p novelsaga-cli` 通过

- [ ] **Task 2.2: 实现 Markdown 链接检测**
  - **文件**: `projects/cli/src/lsp/hover.rs` (追加)
  - **内容**:
    - 函数: `detect_link_at_position(content: &str, position: Position) -> Option<String>`
    - 正则表达式识别 Markdown 链接: `[text](id:scene-123)`
    - 提取链接的 ID
  - **依赖**: Task 2.1 完成
  - **可并行**: ✅ 是 (与 Task 2.3 并行)
  - **验证**: `cargo test -p novelsaga-cli test_detect_markdown_link`

- [ ] **Task 2.3: 实现元数据查询**
  - **文件**: `projects/cli/src/lsp/hover.rs` (追加)
  - **内容**:
    - 函数: `query_metadata(id: &str) -> Option<MetadataInfo>`
    - 通过 MetadataManager 查询 scene/chapter 信息
    - 返回标题、synopsis、字数等
  - **依赖**: Task 2.1 完成, P2 MetadataManager 完成
  - **可并行**: ✅ 是 (与 Task 2.2 并行)
  - **验证**: `cargo test -p novelsaga-cli test_query_metadata`

- [ ] **Task 2.4: 实现 Hover 内容格式化**
  - **文件**: `projects/cli/src/lsp/hover.rs` (追加)
  - **内容**:
    - 函数: `format_hover_content(metadata: &MetadataInfo) -> MarkupContent`
    - 格式化为 Markdown
    - 包含标题、ID、synopsis、字数、状态等
  - **依赖**: Task 2.3 完成
  - **可并行**: ❌ 否 (依赖元数据查询)
  - **验证**: `cargo test -p novelsaga-cli test_format_hover`

- [ ] **Task 2.5: 实现 textDocument/hover 处理**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 实现 `LanguageServer::hover` 方法
    - 从 DocumentStore 获取文档
    - 调用 HoverProvider
    - 返回 Hover 响应
  - **依赖**: Task 1.2, Task 2.4 完成
  - **可并行**: ❌ 否 (依赖 DocumentStore 和 HoverProvider)
  - **验证**: 集成测试 - 悬停在链接上显示元数据

### Phase 3: Completion 功能实现 (自动补全)

- [ ] **Task 3.1: 定义 CompletionProvider trait**
  - **文件**: `projects/cli/src/lsp/completion.rs` (新建)
  - **内容**:
    - `CompletionProvider` trait
    - 方法: `provide_completions(&self, document: &Document, position: Position) -> Vec<CompletionItem>`
  - **依赖**: Task 1.1 完成
  - **可并行**: ✅ 是 (与 Phase 1-2 并行)
  - **验证**: `cargo build -p novelsaga-cli` 通过

- [ ] **Task 3.2: 实现触发字符检测**
  - **文件**: `projects/cli/src/lsp/completion.rs` (追加)
  - **内容**:
    - 函数: `is_completion_trigger(content: &str, position: Position) -> Option<CompletionContext>`
    - 检测触发上下文: `[` (链接开始), `(id:` (ID 前缀)
    - 返回补全类型 (SceneLink, ChapterLink, CharacterName)
  - **依赖**: Task 3.1 完成
  - **可并行**: ✅ 是 (与 Task 3.3-3.4 并行)
  - **验证**: `cargo test -p novelsaga-cli test_completion_trigger`

- [ ] **Task 3.3: 实现 Scene 补全项生成**
  - **文件**: `projects/cli/src/lsp/completion.rs` (追加)
  - **内容**:
    - 函数: `generate_scene_completions() -> Vec<CompletionItem>`
    - 从 MetadataManager 获取所有 scenes
    - 生成补全项: `label = "Scene 1: Title"`, `insertText = "[Scene 1](id:scene-123)"`
    - 添加详情: chapter, synopsis
  - **依赖**: Task 3.1 完成, P2 MetadataManager 完成
  - **可并行**: ✅ 是 (与 Task 3.2, 3.4 并行)
  - **验证**: `cargo test -p novelsaga-cli test_scene_completions`

- [ ] **Task 3.4: 实现 Chapter 补全项生成**
  - **文件**: `projects/cli/src/lsp/completion.rs` (追加)
  - **内容**:
    - 函数: `generate_chapter_completions() -> Vec<CompletionItem>`
    - 从 MetadataManager 获取所有 chapters
    - 生成补全项: `label = "Chapter 1: Title"`, `insertText = "[Chapter 1](id:chapter-123)"`
  - **依赖**: Task 3.1 完成, P2 MetadataManager 完成
  - **可并行**: ✅ 是 (与 Task 3.2-3.3 并行)
  - **验证**: `cargo test -p novelsaga-cli test_chapter_completions`

- [ ] **Task 3.5: 实现 textDocument/completion 处理**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 实现 `LanguageServer::completion` 方法
    - 从 DocumentStore 获取文档
    - 调用 CompletionProvider
    - 返回 CompletionList
  - **依赖**: Task 1.2, Task 3.2-3.4 完成
  - **可并行**: ❌ 否 (依赖所有补全生成器)
  - **验证**: 集成测试 - 输入 `[` 显示补全列表

- [ ] **Task 3.6: 实现 completionItem/resolve (可选增强)**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 实现 `LanguageServer::completion_resolve` 方法
    - 延迟加载补全项的详细信息 (synopsis)
    - 优化大型项目的性能
  - **依赖**: Task 3.5 完成
  - **可并行**: ❌ 否 (依赖基础补全)
  - **验证**: 集成测试 - 选择补全项后显示详情

### Phase 4: LSP 服务器配置和注册

- [ ] **Task 4.1: 定义服务器能力 (ServerCapabilities)**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 在 `initialize` 方法中配置 `ServerCapabilities`
    - 启用: `text_document_sync`, `hover_provider`, `completion_provider`
    - 配置补全触发字符: `[`, `(`
  - **依赖**: Task 2.5, Task 3.5 完成
  - **可并行**: ❌ 否 (依赖功能实现)
  - **验证**: LSP client 识别服务器能力

- [ ] **Task 4.2: 添加 DocumentStore 到 Backend**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 在 `Backend` 结构体中添加 `document_store: Arc<DocumentStore>` 字段
    - 在构造函数中初始化
  - **依赖**: Task 1.2 完成
  - **可并行**: ❌ 否 (依赖 DocumentStore)
  - **验证**: `cargo build -p novelsaga-cli` 通过

- [ ] **Task 4.3: 添加 MetadataManager 访问**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 在 `Backend` 结构体中添加 `metadata_manager: Arc<MetadataManager>` 字段
    - 或者通过 `Initializer::get()` 访问全局 State
  - **依赖**: P2 MetadataManager 集成到 State 完成
  - **可并行**: ❌ 否 (依赖 P2)
  - **验证**: `cargo build -p novelsaga-cli` 通过

### Phase 5: 诊断功能 (Diagnostics) - 可选

- [ ] **Task 5.1: 实现损坏链接检测**
  - **文件**: `projects/cli/src/lsp/diagnostics.rs` (新建)
  - **内容**:
    - 函数: `check_broken_links(document: &Document) -> Vec<Diagnostic>`
    - 扫描文档中的所有链接
    - 检查元数据是否存在
    - 生成错误诊断 (红色波浪线)
  - **依赖**: Task 1.2, P2 MetadataManager 完成
  - **可并行**: ✅ 是 (独立功能)
  - **验证**: `cargo test -p novelsaga-cli test_broken_links`

- [ ] **Task 5.2: 实现 textDocument/publishDiagnostics**
  - **文件**: `projects/cli/src/lsp/backend.rs` (修改)
  - **内容**:
    - 在 `did_open`, `did_change`, `did_save` 中调用诊断
    - 通过 `client.publish_diagnostics()` 发送诊断
  - **依赖**: Task 5.1 完成
  - **可并行**: ❌ 否 (依赖诊断生成)
  - **验证**: 集成测试 - 损坏链接显示错误

### Phase 6: 集成测试和文档

- [ ] **Task 6.1: 编写 LSP 集成测试**
  - **文件**: `projects/cli/tests/lsp_integration.rs` (新建)
  - **内容**:
    - 启动 LSP 服务器
    - 模拟 LSP 客户端请求
    - 测试 hover, completion, diagnostics
    - 使用 tempfile 创建测试项目
  - **依赖**: Task 4.3 完成 (所有 LSP 功能完成)
  - **可并行**: ❌ 否 (依赖所有功能)
  - **验证**: `cargo test -p novelsaga-cli test_lsp_integration`

- [ ] **Task 6.2: 更新 CLI AGENTS.md**
  - **文件**: `projects/cli/AGENTS.md` (修改)
  - **内容**:
    - 添加 LSP 功能文档
    - 描述 hover, completion, diagnostics
    - 添加使用示例
  - **依赖**: Task 6.1 完成
  - **可并行**: ❌ 否 (依赖完整实现)
  - **验证**: 文档完整性检查

- [ ] **Task 6.3: 创建 LSP 配置示例**
  - **文件**: `docs/lsp-config-examples.md` (新建)
  - **内容**:
    - VSCode 配置示例
    - Neovim 配置示例
    - Helix 配置示例
    - 调试技巧
  - **依赖**: Task 6.1 完成
  - **可并行**: ✅ 是 (与 Task 6.2 并行)
  - **验证**: 文档可读性检查

### Phase 7: 性能优化 (可选)

- [ ] **Task 7.1: 实现增量文档解析**
  - **文件**: `projects/cli/src/lsp/document.rs` (修改)
  - **内容**:
    - 优化 `did_change` 处理
    - 仅重新解析变更的行
    - 缓存解析结果
  - **依赖**: Task 1.4 完成
  - **可并行**: ✅ 是 (性能优化)
  - **验证**: 性能基准测试 - 大文件编辑延迟 <50ms

- [ ] **Task 7.2: 实现补全缓存**
  - **文件**: `projects/cli/src/lsp/completion.rs` (修改)
  - **内容**:
    - 缓存补全项列表
    - 仅在元数据变更时刷新
    - 使用 dirty flag 跟踪
  - **依赖**: Task 3.5 完成
  - **可并行**: ✅ 是 (性能优化)
  - **验证**: 性能基准测试 - 补全响应 <10ms

## Parallelization Map

**可并行的任务组**:

**Group 1 (Phase 1 文档生命周期)**:

- Task 1.3 (didOpen)
- Task 1.4 (didChange)
- Task 1.5 (didSave)
- Task 1.6 (didClose)
  这四个可以同时进行（都依赖 DocumentStore）。

**Group 2 (Phase 2 Hover)**:

- Task 2.2 (链接检测)
- Task 2.3 (元数据查询)
  这两个可以同时进行。

**Group 3 (Phase 3 Completion)**:

- Task 3.2 (触发检测)
- Task 3.3 (Scene 补全)
- Task 3.4 (Chapter 补全)
  这三个可以同时进行。

**Group 4 (Phase 6 文档)**:

- Task 6.2 (AGENTS.md)
- Task 6.3 (配置示例)
  这两个可以同时进行。

**Group 5 (Phase 7 优化)**:

- Task 7.1 (增量解析)
- Task 7.2 (补全缓存)
  这两个可以同时进行。

## Estimated Timeline

| Phase     | Tasks        | Estimated Time | Dependencies       |
| --------- | ------------ | -------------- | ------------------ |
| Phase 1   | Task 1.1-1.6 | 1.5 天         | 无                 |
| Phase 2   | Task 2.1-2.5 | 1.5 天         | Phase 1, P2        |
| Phase 3   | Task 3.1-3.6 | 2 天           | Phase 1, P2        |
| Phase 4   | Task 4.1-4.3 | 0.5 天         | Phase 2, 3         |
| Phase 5   | Task 5.1-5.2 | 1 天           | Phase 1, P2 (可选) |
| Phase 6   | Task 6.1-6.3 | 1 天           | Phase 4            |
| Phase 7   | Task 7.1-7.2 | 0.5 天         | Phase 3 (可选)     |
| **Total** | **28 tasks** | **7 天**       |                    |

## Critical Dependencies

**必须先完成的外部任务**:

- **P2 MetadataManager** (Phase 4, Task 4.1-4.7) - 必须先完成才能实现 hover 和 completion
- **P2 State 集成** (Phase 6, Task 6.1-6.3) - 必须先完成才能在 LSP backend 中访问元数据

**建议顺序**:

1. 完成 P2 MetadataManager 的 Phase 1-4 (核心 CRUD)
2. 开始 P3 的 Phase 1 (文档生命周期) - 可与 P2 Phase 5-6 并行
3. 完成 P2 State 集成
4. 继续 P3 的 Phase 2-4 (Hover + Completion)

## Verification Checklist

每个任务完成后必须验证:

- [ ] `cargo build -p novelsaga-cli` 通过
- [ ] `cargo test -p novelsaga-cli` 全部通过
- [ ] `cargo clippy --all-targets` 0 warnings
- [ ] `lsp_diagnostics` 0 errors
- [ ] LSP 客户端可以正常连接和通信
- [ ] 任务在 plan 中标记为完成

## Testing Strategy

**单元测试** (每个功能模块):

- `test_document_store` - DocumentStore CRUD
- `test_detect_markdown_link` - 链接检测
- `test_query_metadata` - 元数据查询
- `test_completion_trigger` - 补全触发
- `test_scene_completions` - Scene 补全生成

**集成测试** (LSP 协议):

- `test_lsp_hover` - 悬停在链接上显示元数据
- `test_lsp_completion` - 输入触发字符显示补全
- `test_lsp_diagnostics` - 损坏链接显示错误
- `test_lsp_lifecycle` - 打开/修改/保存/关闭文档

**手动测试** (真实编辑器):

- VSCode + NovelSaga LSP
- Neovim + NovelSaga LSP
- 测试大型项目性能 (1000+ 场景)

## Notes

- **MVP**: 完成 Phase 1-4, 6 即可发布 (核心功能)
- **Enhancement**: Phase 5 (诊断) 和 Phase 7 (优化) 可后续添加
- **P2 依赖**: 必须先完成 P2 MetadataManager 的核心功能
- **异步处理**: 所有 LSP 方法都是异步的，注意使用 `tokio::spawn` 和 `Arc`
- **错误处理**: LSP 错误不应导致服务器崩溃，使用 `log::error!` 记录
