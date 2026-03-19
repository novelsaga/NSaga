# P3 - LSP 服务器集成设计概要

> **状态更新 (2026-02-27)**: ✅ **P2 已完成，阻塞解除，准备开始**
> - ✅ **P2 元数据管理器已全部完成** (2026-02-27)
> - **关键调整**: LSP 优先使用 hooks (didOpen/didChange/didSave)，`FileWatcher` 降级备用
> - 可用组件: `IndexManager`, `CacheManager`, `WriteBackWorker` (P2 已完成)
> - ✅ **解阻塞**: CLI 命令 (index/list/show) 已实现，CLI-to-LSP 路由已就绪

## 概述

LSP 服务器功能集成的设计规划，包含文档管理、hover、completion 和诊断功能。

> **注意**: 此为设计概要文档，代码实现细节可能已过时。执行前请验证当前代码结构。

## 架构设计

### LSP 文件变更处理策略

| 方案 | 实现方式 | 优先级 | 说明 |
|------|---------|--------|------|
| **LSP Hooks** | `didOpen`/`didChange`/`didSave`/`didClose` | **首选** | 与编辑器生命周期绑定，更精确 |
| FileWatcher | notify 库监听文件系统 | 降级备用 | P2 已实现，但 hooks 更可靠 |

**决策**: 优先使用 LSP hooks，FileWatcher 作为无 LSP 客户端时的降级方案。

### LSP 组件栈

```rust
pub struct LspBackend {
    // P2 提供的组件
    index: Arc<IndexManager>,      // sled 持久化
    cache: CacheManager,           // moka 热缓存
    worker: WriteBackWorker,       // flume + tokio 异步写回
    
    // LSP 文档管理
    documents: Arc<RwLock<BTreeMap<Url, Document>>>,
}

// 文件变更流
LSP Hooks (didOpen/didChange/didSave)
    ↓
更新 Document 内存状态
    ↓
解析为 MetadataEntity
    ↓
更新 CacheManager (热缓存)
    ↓
提交 WriteTask 到 WriteBackWorker
    ↓
异步批量写入 IndexManager → sled
```

## 功能目标

1. **文档生命周期**: didOpen, didChange, didSave, didClose
2. **Hover**: 悬停显示元数据信息（从 cache 读取）
3. **Completion**: 场景/章节自动补全
4. **Diagnostics**: 损坏链接检测 (可选)

## 现有架构

- LSP backend 骨架: `projects/cli/src/lsp/backend.rs`
- 使用 `tower-lsp` crate
- 异步处理 (`tokio`)
- **P2 可用组件**:
  - `IndexManager`: sled 持久化索引
  - `CacheManager`: moka 内存缓存
  - `WriteBackWorker`: flume + tokio 异步写回
  - `FileWatcher`: notify 文件监听（降级备用）

## 实现阶段

| Phase | 内容               | 任务数 | 预计时间 | 依赖 |
| ----- | ------------------ | ------ | -------- | ---- |
| Phase 1 | 文档生命周期管理   | 6      | 1.5 天   | P2 IndexManager |
| Phase 2 | Hover 功能         | 5      | 1.5 天   | P2 CacheManager |
| Phase 3 | Completion 功能    | 6      | 2 天     | P2 查询接口 |
| Phase 4 | 服务器配置和注册   | 3      | 0.5 天   | - |
| Phase 5 | Diagnostics (可选) | 2      | 1 天     | - |
| Phase 6 | 集成测试和文档     | 3      | 1 天     | - |
| Phase 7 | 性能优化 (可选)    | 2      | 0.5 天   | - |
| **合计** |                    | **28** | **7 天** | |

## 关键设计决策

- **DocumentStore**: `Arc<RwLock<BTreeMap<Url, Document>>>`
- **文件变更**: 优先使用 LSP hooks，FileWatcher 备用
- **数据流**: Hooks → Document → MetadataEntity → Cache → Worker → Index
- **查询路径**: Hover/Completion 优先读 Cache（热数据），miss 时读 Index
- **持久化**: WriteBackWorker 批量异步写入 sled

## MVP 范围

完成 Phase 1-4, 6 即可发布核心功能。Phase 5 (诊断) 和 Phase 7 (优化) 可后续添加。

## 依赖关系

**必须先完成**:

- ✅ P2 MetadataManager Core (`MetadataEntity`, `MetadataQuery`, parser)
- ✅ P2 IndexManager (sled 索引)
- ✅ P2 CacheManager (moka 缓存)
- ✅ P2 WriteBackWorker (异步写回)
- ⏳ P2 CLI 命令实现 (Phase 7)

## 当前阻塞状态 (2026-02-27)

| 依赖项 | 状态 | 说明 |
|--------|------|------|
| P2 Core (Entity, Query, Parser) | ✅ 完成 | 数据模型和接口 |
| P2 IndexManager | ✅ 完成 | sled 持久化 |
| P2 CacheManager | ✅ 完成 | moka 热缓存 |
| P2 WriteBackWorker | ✅ 完成 | 异步写回 |
| P2 FileWatcher | ✅ 完成 | notify 监听（LSP 中可选） |
| **P2 CLI 命令** | ⏳ **阻塞中** | `index`/`list`/`show` 待实现 |

### 解阻塞条件

P2 完成 CLI 命令后，P3 可立即开始：

1. **P2 Phase 7 完成** (`projects/cli/src/commands/metadata.rs`)
   - `handle_index()` - 目录索引
   - `handle_list()` - 列出 metadata
   - `handle_show()` - 显示详情

### P3 启动时的组件状态

P3 开始时，以下组件已就绪可直接使用：

```rust
// 可以直接使用的 P2 组件
use novelsaga_cli::metadata::index::IndexManager;
use novelsaga_cli::metadata::cache::CacheManager;
use novelsaga_cli::metadata::worker::{WriteBackWorker, WriteTask};

// 可选（如 LSP hooks 不够用时）
use novelsaga_cli::metadata::watcher::{FileWatcher, FileChangeEvent};
```

## P3 实现要点

### 1. 文档生命周期 (Phase 1)

```rust
impl LanguageServer for LspBackend {
    async fn did_open(&self, params: DidOpenTextDocumentParams,
    ) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        
        // 1. 存入 DocumentStore
        let mut docs = self.documents.write().await;
        docs.insert(uri.clone(), Document::new(uri.clone(), text));
        
        // 2. 解析为 MetadataEntity
        if is_metadata_file(&uri) {
            let entity = parse_metadata(&text, &uri).await;
            
            // 3. 更新缓存
            self.cache.insert(entity.id.clone(), entity.clone()).await;
            
            // 4. 提交异步写回
            self.worker.submit(WriteTask::Upsert {
                id: entity.id,
                data: serde_json::to_vec(&entity).unwrap(),
            });
        }
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams,
    ) {
        // 类似 did_open，更新内存文档和缓存
    }
    
    async fn did_save(&self, params: DidSaveTextDocumentParams,
    ) {
        // 触发 WriteBackWorker flush（可选）
        self.worker.submit(WriteTask::Flush);
    }
}
```

### 2. Hover 功能 (Phase 2)

```rust
async fn hover(&self, params: HoverParams,
) -> tower_lsp::jsonrpc::Result<Option<Hover>> {
    let position = params.text_document_position_params.position;
    let uri = params.text_document_position_params.text_document.uri;
    
    // 1. 从 DocumentStore 获取当前文档
    let docs = self.documents.read().await;
    let doc = docs.get(&uri).ok_or_else(|| ...)?;
    
    // 2. 解析光标位置的链接/引用
    let ref_id = doc.extract_reference_at(position)?;
    
    // 3. 从 Cache 查询（热数据）
    if let Some(entity) = self.cache.get(&ref_id).await {
        return Ok(Some(format_hover(entity)));
    }
    
    // 4. Cache miss，从 IndexManager 查询
    if let Some(entity) = self.index.get_by_id(&ref_id)? {
        // 回填 cache
        self.cache.insert(ref_id, entity.clone()).await;
        return Ok(Some(format_hover(entity)));
    }
    
    Ok(None)
}
```

### 3. FileWatcher 降级方案

如果某些 LSP 客户端不发送完整的 didChange，可以启用 FileWatcher：

```rust
impl LspBackend {
    async fn start_file_watcher(&self, 
        workspace_root: PathBuf
    ) -> anyhow::Result<()> {
        // 可选：仅在需要时启动
        let mut watcher = FileWatcher::new(&workspace_root)?;
        watcher.watch(&workspace_root)?;
        
        tokio::spawn(async move {
            loop {
                if let Some(event) = watcher.try_recv() {
                    match event {
                        FileChangeEvent::Modified(path) => {
                            // 重新加载文件并更新索引
                            self.reload_file(&path).await;
                        }
                        // ...
                    }
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        
        Ok(())
    }
}
```

详细进度见：`p2-metadata-manager-breakdown.md`
