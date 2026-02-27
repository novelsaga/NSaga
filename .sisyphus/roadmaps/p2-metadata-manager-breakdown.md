# P2 - 元数据管理器设计概要

> **状态更新 (2026-02-27)**: ✅ **全部完成**
> - ✅ Phase 1-2: Core 数据模型、查询接口
> - ✅ Phase 4: `IndexManager` (sled) - **CLI 直接可用**
> - ✅ Phase 3,5,6: `CacheManager`/`FileWatcher`/`WriteBackWorker` - **仅供 LSP 使用**
> - ✅ Phase 7: CLI 命令实现（index/list/show）+ CLI-to-LSP 路由

---

## 概述

小说元数据管理器的设计规划，基于文件系统的元数据管理，使用 Sled 嵌入式 KV 存储作为索引。

## 架构调整说明

### 原设计 vs 调整后

| 组件 | 原设计用途 | 调整后用途 | 原因 |
|------|-----------|-----------|------|
| `IndexManager` | CLI + LSP | ✅ **CLI + LSP 共用** | 基础持久化层 |
| `CacheManager` | CLI + LSP | ⚠️ **仅 LSP** | CLI 命令是一次性的，不需要缓存 |
| `FileWatcher` | CLI + LSP | ⚠️ **仅 LSP (可选)** | CLI 用一次性扫描；LSP 可用 hooks 替代 |
| `WriteBackWorker` | CLI + LSP | ⚠️ **仅 LSP** | CLI 批量写入直接调用 IndexManager |

### CLI vs LSP 架构差异

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI 模式 (简化)                          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │   index cmd │───▶│ IndexManager│───▶│    sled     │         │
│  │  (批量扫描)  │    │  (直接写入)  │    │  (持久存储)  │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│                                                                  │
│  ┌─────────────┐    ┌─────────────┐                             │
│  │  list/show  │───▶│ IndexManager│───▶ 查询 sled              │
│  │  (查询命令)  │    │  (直接读取)  │                             │
│  └─────────────┘    └─────────────┘                             │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                     LSP 模式 (完整)                              │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │  LSP hooks  │    │ WriteBack   │    │ IndexManager│         │
│  │didOpen/     │───▶│   Worker    │───▶│  (批量写入)  │         │
│  │didChange/   │    │ (flume+tokio)│    └──────┬──────┘         │
│  │didSave      │    └─────────────┘           │                │
│  └─────────────┘                              ▼                │
│                                       ┌─────────────┐         │
│  ┌─────────────┐    ┌─────────────┐   │    sled     │         │
│  │ CacheManager│◀───│  热数据缓存  │◀──┘  (持久存储)  │         │
│  │   (moka)    │    └─────────────┘                      │
│  └─────────────┘                                              │
│                                                                  │
│  注: FileWatcher 可能不需要，LSP hooks 更精确                     │
└─────────────────────────────────────────────────────────────────┘
```

## 分层架构

```
projects/
├── core/                          # 核心逻辑（无 IO）
│   └── src/metadata/
│       ├── model.rs               # 数据模型（MetadataEntity）
│       ├── parser.rs              # 类型推导、路径解析
│       └── query.rs               # 查询接口定义（trait）
│
└── cli/                           # IO 层实现
    └── src/metadata/
        ├── index.rs                 # sled 索引操作 [CLI+LSP共用]
        ├── cache.rs                 # moka 内存缓存 [仅LSP]
        ├── watcher.rs               # notify 文件监听 [仅LSP，可选]
        └── worker.rs                # 异步写回 Worker [仅LSP]
```

**设计原则**: 
- Core 纯逻辑，CLI 处理所有 IO
- CLI 命令使用简化架构（直接读写 IndexManager）
- LSP 使用完整架构（异步+缓存+批量写入）

## 存储架构

- **元数据文件**: `metadata/` 目录下的 Markdown 文件（frontmatter + 正文）
- **索引**: `.cache/novelsaga/` 下的 Sled 数据库（可重建，可 gitignore）
- **类型推导**: 路径推断（`metadata/characters/hero.md` → type: character）+ frontmatter 覆盖

## 数据模型（core）

```rust
// 纯数据结构，无 IO
struct MetadataEntity {
    id: String,
    type_: String,
    namespace: String,
    frontmatter: serde_json::Value,
    body: String,
}

// 查询接口（core 定义，cli 实现）
trait MetadataQuery {
    fn get(&self, name: &str) -> Option<MetadataEntity>;
    fn list_by_type(&self, type_: &str) -> Vec<MetadataEntity>;
}
```

## 缓存策略（LSP 专用）

**Write-Back 模式**: 内存同步修改，磁盘异步落盘

- moka 热缓存（LRU，容量可配置）
- 后台 Worker 批量写入 sled（flume 通道）
- 文件监听/失效：优先使用 LSP hooks，FileWatcher 备用

## Type 推导规则

| 文件路径                     | 默认 Type   |
| ---------------------------- | ----------- |
| `metadata/xxx.md`            | `metadata`  |
| `metadata/characters/xxx.md` | `character` |
| `metadata/scenes/xxx.md`     | `scene`     |
| `metadata/locations/xxx.md`  | `location`  |

**优先级**: frontmatter `type` 字段 > 路径推断

## Sled Key 设计

```
entity:<id>          # 实体数据 (JSON)
name:<ns>:<name>    # name → id 映射 (用于按名查找)
type:<type>:<id>    # type → id 索引
ns:<ns>:<id>       # namespace → id 索引
```

## 实现阶段

| Phase | 模块 | 内容                 | 任务数 | 预计时间 | 状态 | 用途 |
| ----- | ---- | -------------------- | ------ | -------- | ---- | ---- |
| 1     | core | 数据模型、类型推导器 | 3      | 0.5 天   | ✅   | CLI+LSP |
| 2     | core | 查询接口定义         | 2      | 0.25 天  | ✅   | CLI+LSP |
| 3     | cli  | moka 缓存层          | 3      | 0.5 天   | ✅   | **仅 LSP** |
| 4     | cli  | sled 索引层          | 3      | 0.5 天   | ✅   | **CLI+LSP** |
| 5     | cli  | notify 文件监听      | 3      | 0.5 天   | ✅   | **仅 LSP (可选)** |
| 6     | cli  | 异步写回 Worker      | 3      | 0.5 天   | ✅   | **仅 LSP** |
| 7     | cli  | **CLI 命令简化实现** | 2      | 0.5 天   | ⏳   | CLI 直接用 IndexManager |
| 8     | -    | 集成测试             | 2      | 0.5 天   | ⏳   | CLI 命令测试 |
| **合计** |      |                      | **21** | **3.75 天** |      | |

## 关键设计决策

- **分层**: core 纯逻辑，cli 处理所有 IO
- **架构分层**: CLI 命令简化（直接读写），LSP 完整（异步+缓存）
- **存储分离**: 文件（真相源）+ sled（索引）+ moka（LSP 热缓存）
- **监听策略**: LSP 优先使用 hooks，FileWatcher 降级备用

## NovelSaga Workspace 结构要求

### 目录布局（层级不封顶）

约定：任意目录下可放置 `metadata/` 文件夹，该目录即成为一个 scope。scope 可以无限嵌套。

```
workspace/                               # 工作空间（任意名称）
├── .cache/novelsaga/                    # 缓存
│   └── sled/                           # 索引数据库
│
├── novelsaga.config.toml               # 项目配置
├── .novelsagaignore                    # 忽略规则
│
├── metadata/                           # 根级 scope
│   ├── hero.md                         # type: metadata（默认）
│   ├── characters/
│   │   ├── protagonist.md              # type: character
│   │   └── villain.md
│   └── locations/
│       └── shanghai.md                 # type: location
│
└── book-01/                            # 作品级（可选，可多级）
    ├── metadata/
    │   └── hero.md                     # 覆盖全局的 hero（成长/变化）
    ├── part-01/
    │   ├── metadata/
    │   └── chapter-01/
    │       ├── metadata/
    │       │   └── temp-guard.md       # 仅本章可见
    │       └── scene-01.md             # 正文（无元数据）
    └── ...
```

**核心规则**:
- 任意目录下出现 `metadata/` 即成为一个独立的 scope
- scope 层级无限制（上不封顶）
- 作者自由组织目录结构，无需遵循固定层级名称（book/part/chapter 只是示例）

### 就近解析规则

从当前文件位置向上遍历，遇到的每个 `metadata/` 都是一个 scope，优先级**最近优先**。

**示例**：编辑 `book-01/part-01/chapter-01/scene-01.md` 时查找 `hero`：
1. `book-01/part-01/chapter-01/metadata/hero.md`（最近）
2. `book-01/part-01/metadata/hero.md`
3. `book-01/metadata/hero.md`
4. `metadata/hero.md`（最远）

**Namespace 生成**：从工作空间根到 `metadata/` 父目录的相对路径
- `metadata/` → `global`
- `book-01/metadata/` → `book-01`
- `book-01/part-01/chapter-01/metadata/` → `book-01/part-01/chapter-01`

| 层级 | 路径模式 | Namespace |
|------|----------|-----------|
| 全局 | `metadata/` | `global` |
| 作品 | `book-*/metadata/` | `book-*` |
| 卷 | `part-*/metadata/` | `book-*/part-*` |
| 章 | `chapter-*/metadata/` | `book-*/part-*/chapter-*` |

### 类型推导（文件名 → type）

| 文件路径 | 推导 Type |
|----------|-----------|
| `metadata/*.md` | `metadata` |
| `metadata/characters/*.md` | `character` |
| `metadata/scenes/*.md` | `scene` |
| `metadata/locations/*.md` | `location` |
| `metadata/items/*.md` | `item` |
| ... | 文件夹名单数化 |

**覆盖**: frontmatter 中的 `type` 字段优先于路径推导。

## CLI 命令（简化实现）

```bash
novelsaga index              # 一次性扫描并建立索引
novelsaga list --type character --namespace book-01
novelsaga show hero          # 显示元数据详情
```

**注**: `index --watch` 模式不再需要，LSP 会实时处理变更。

## 依赖

| 模块 | 依赖                | 用途     | 使用方 |
| ---- | ------------------- | -------- | ------ |
| core | serde               | 序列化   | CLI+LSP |
| cli  | sled                | 持久索引 | CLI+LSP |
| cli  | moka                | 内存缓存 | **仅 LSP** |
| cli  | notify              | 文件监听 | **仅 LSP (可选)** |
| cli  | flume/async-channel | 异步通道 | **仅 LSP** |

## 实现状态详情

### ✅ 已完成 (Phase 1-6)

| 组件 | 文件路径 | 说明 | 使用方 |
|------|----------|------|--------|
| `MetadataEntity` | `projects/core/src/metadata/model.rs` | 数据模型，TS 导出 | CLI+LSP |
| 类型推导器 | `projects/core/src/metadata/parser.rs` | 路径推断、namespace 生成 | CLI+LSP |
| 查询接口 | `projects/core/src/metadata/query.rs` | `MetadataQuery` trait + `InMemoryMetadataStore` | CLI+LSP |
| 索引管理器 | `projects/cli/src/metadata/index.rs` | `IndexManager` (sled)，二级索引 | **CLI+LSP** |
| 缓存管理器 | `projects/cli/src/metadata/cache.rs` | `CacheManager` (moka) | **仅 LSP** |
| 文件监听 | `projects/cli/src/metadata/watcher.rs` | `FileWatcher` (notify) | **仅 LSP (可选)** |
| 异步写回 | `projects/cli/src/metadata/worker.rs` | `WriteBackWorker` (flume + tokio) | **仅 LSP** |

### ⏳ 待完成 (Phase 7-8)

| 任务 | 文件路径 | 说明 | 架构 |
|------|----------|------|------|
| CLI 命令处理器 | `projects/cli/src/commands/metadata.rs` | `index`/`list`/`show` 为 `todo!()` | **简化架构** |
| 集成测试 | `projects/cli/tests/` | CLI 命令测试 | **简化架构** |

### 📝 CLI 简化实现方案

Phase 7 不需要复杂的组件连接：

```rust
// handle_index - 直接使用 IndexManager
async fn handle_index(cmd: IndexCommand) -> anyhow::Result<()> {
    let index = IndexManager::open(cache_dir)?;
    
    // 遍历所有 metadata/*.md 文件
    for entry in walkdir(&cmd.path).filter(|e| is_metadata_file(e)) {
        let entity = parse_and_create_entity(entry.path())?;
        index.index_entity(&entity)?;  // 直接写入，无需 worker
    }
    
    index.flush()?;
    println!("Indexed {} files", count);
    Ok(())
}

// handle_list - 直接查询 sled
async fn handle_list(cmd: ListCommand) -> anyhow::Result<()> {
    let index = IndexManager::open(cache_dir)?;
    
    let entities = match cmd.type_ {
        Some(t) => index.list_by_type(&t)?,
        None => index.list_all()?,  // 需要添加此方法
    };
    
    print_metadata_table(&entities);
    Ok(())
}

// handle_show - 直接查询
async fn handle_show(cmd: ShowCommand) -> anyhow::Result<()> {
    let index = IndexManager::open(cache_dir)?;
    let entity = find_by_name(&index, &cmd.name, &current_ns)?;
    println!("{}", serde_json::to_string_pretty(&entity)?);
    Ok(())
}
```

## 阻塞影响

- **P3 LSP 集成**: 被阻塞，因为 LSP 依赖 metadata 查询功能
- **前置条件**: P2 Phase 7-8 完成后才能开始 P3
- **P3 可用组件**: CacheManager、WriteBackWorker、FileWatcher（如需要）

## P3 LSP 集成时的组件使用

```rust
// P3 LSP Backend
pub struct LspBackend {
    index: Arc<IndexManager>,           // P2 提供
    cache: CacheManager,                // P2 提供
    worker: WriteBackWorker,            // P2 提供
    // FileWatcher 可能不需要，用 LSP hooks 替代
}

impl LanguageServer for LspBackend {
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        // 1. 解析文件
        let entity = parse_document(&params.text_document.uri)?;
        
        // 2. 更新缓存
        self.cache.insert(entity.id.clone(), entity.clone()).await;
        
        // 3. 提交异步写回
        self.worker.submit(WriteTask::Upsert {
            id: entity.id,
            data: serde_json::to_vec(&entity)?,
        });
    }
    
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // 类似 did_open，更新缓存并提交 worker
    }
    
    async fn hover(&self, params: HoverParams) -> Option<Hover> {
        // 从 cache 查询（热数据）
        let entity = self.cache.get(&id).await?;
        Some(format_hover(&entity))
    }
}
```

## 下一步行动

### Phase 7: CLI 命令简化实现

1. **实现 `handle_index`** (`projects/cli/src/commands/metadata.rs`)
   - 创建 `IndexManager` 实例
   - 遍历目录，解析 `metadata/*.md` 文件
   - 直接调用 `index.index_entity()`（无需 worker）
   - 完成后 `flush()`

2. **实现 `handle_list`**
   - 打开 `IndexManager`
   - 根据 `--type` 调用 `list_by_type()` 或 `list_all()`
   - 格式化输出表格

3. **实现 `handle_show`**
   - 打开 `IndexManager`
   - 通过 name + namespace 查找（需要添加 `find_by_name`）
   - 格式化输出（JSON 或表格）

4. **导出组件** (`projects/cli/src/metadata/mod.rs`)
   - 取消注释 `pub use index::IndexManager;`
   - 其他组件保持注释（仅供 LSP 使用）

### Phase 8: 集成测试

- 测试 `index` 命令：扫描目录 → 验证 sled 数据库
- 测试 `list` 命令：查询不同类型的实体
- 测试 `show` 命令：按名查找实体
- 测试就近解析：同名实体在不同 namespace 的优先级

### P3 准备

- 验证 `CacheManager`、`WriteBackWorker` 接口
- 评估是否需要 `FileWatcher`（建议优先使用 LSP hooks）
- 规划 LSP 与 CLI 共享 sled 数据库的并发策略


## 架构审查记录 (2026-02-27)

### 审查结论

✅ **整体架构合理**，Core/CLI 分层清晰，组件选型成熟（sled/moka/flume/notify）。
✅ **CLI/LSP 分离策略正确**：CLI 简化、LSP 完整的方向是对的。
⚠️ **发现 2 个 CRITICAL 问题** 需在 Phase 7 前修复或设计。

---

### 🔴 CRITICAL-1: `index_entity()` 更新时不清除旧二级索引

**问题描述**
当 entity 的 `type_` 或 `namespace` 变更后重新索引，新索引被插入，但**旧索引永远不会被删除**，导致 `list_by_type()` 返回幽灵数据。

**复现场景**
```
1. hero.md: type=character → index_entity()
   → 写入索引: type:character:hero

2. 用户修改 hero.md，type 改为 npc

3. 重新 index_entity()
   → 写入新索引: type:npc:hero
   → type:character:hero 仍然存在（幽灵条目）

4. novelsaga list --type character → 错误地返回 hero
```

**影响**
- 分类查询返回过期数据
- 数据量越大，幽灵条目越多
- 用户体验：看到已不存在的分类关系

**修复方案**
```rust
// 在 index_entity() 开头添加
pub fn index_entity(&self, entity: &MetadataEntity) -> Result<(), sled::Error> {
    // 先清除旧的二级索引（如果 entity 已存在）
    self.remove_entity(&entity.id)?;  // ← 添加这行
    
    // 然后写入新的 entity + 所有索引
    // ...
}
```

**修复成本**: 极低（1 行代码），`remove_entity()` 已实现完整清理逻辑。

---

### 🔴 CRITICAL-2: sled 多进程访问冲突

**问题描述**
sled 不支持多进程同时打开同一数据库。LSP 运行时执行 CLI 命令会失败。

**复现场景**
```
1. VSCode 打开项目 → LSP 启动 → sled::open() 成功

2. 用户在终端运行 novelsaga index
   → 试图再次 sled::open() 同一目录
   → 返回 Io Error: Resource busy
```

**用户决策**: **方案 A - CLI 路由到 LSP**

**方案 A 详细设计**

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI 命令执行流程                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  novelsaga index /path                                     │
│       │                                                     │
│       ▼                                                     │
│  ┌─────────────┐                                            │
│  │ 检测 LSP    │─── LSP 运行? ──Yes──┐                      │
│  │ 是否运行    │                     │                      │
│  └─────────────┘                     │                      │
│       │ No                           │                      │
│       ▼                              ▼                      │
│  ┌─────────────┐              ┌─────────────┐              │
│  │ 直接操作    │              │ 通过 LSP    │              │
│  │ IndexManager│              │ JSON-RPC    │              │
│  │ (sled)      │              │ 发送命令    │              │
│  └─────────────┘              └──────┬──────┘              │
│                                      │                      │
│                                      ▼                      │
│                              ┌─────────────┐              │
│                              │ LSP Backend │              │
│                              │ 执行操作    │              │
│                              │ 返回结果    │              │
│                              └─────────────┘              │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**实现要点**

1. **LSP 端**: 暴露 JSON-RPC 方法供 CLI 调用
```rust
// projects/cli/src/lsp/backend.rs
impl LspBackend {
    // CLI 通过 JSON-RPC 调用的方法
    pub async fn cli_index(&self, path: String) -> Result<IndexResult, RpcError> {
        // 使用 IndexManager 执行索引
        // 返回统计信息（文件数、entity 数）
    }
    
    pub async fn cli_list(&self, type_filter: Option<String>) -> Result<Vec<EntitySummary>, RpcError> {
        // 查询 sled 并返回列表
    }
    
    pub async fn cli_show(&self, name: String, namespace: String) -> Result<Option<MetadataEntity>, RpcError> {
        // 按 name + namespace 查找
    }
}
```

2. **CLI 端**: 检测 LSP 并路由
```rust
// projects/cli/src/commands/metadata.rs
async fn handle_index(cmd: IndexCommand) -> anyhow::Result<()> {
    // 检测 LSP 是否运行（通过 pid 文件或 socket）
    if let Some(lsp_client) = LspClient::connect().await? {
        // LSP 在运行，通过 JSON-RPC 路由
        let result = lsp_client.cli_index(cmd.path.to_string()).await?;
        println!("Indexed {} files, {} entities", result.file_count, result.entity_count);
    } else {
        // LSP 未运行，直接操作 sled
        let index = IndexManager::open(cache_dir)?;
        // ... 直接索引
    }
}
```

3. **通信方式**
- LSP 启动时在 `.cache/novelsaga/lsp.sock` 创建 Unix socket（或 Windows named pipe）
- CLI 尝试连接该 socket
- 使用 JSON-RPC 2.0 协议通信
- 如果连接失败，视为 LSP 未运行，直接操作 sled

**优势**
- 避免 sled 多进程冲突
- CLI 和 LSP 共享相同的索引逻辑
- LSP 可以预热缓存（index 完成后 entity 已在 cache）

**复杂度**
- 中等：需要实现简单的 JSON-RPC 客户端/服务器
- 需要处理 LSP 启动/关闭时的连接状态

---

### 🟡 HIGH 优先级问题（记录供后续优化）

| 问题 | 影响 | 建议修复时机 |
|------|------|------------|
| **HIGH-1**: `IndexManager` 未实现 `MetadataQuery` trait（签名不匹配） | 无法对内存存储和持久存储做多态替换 | P3 或重构阶段 |
| **HIGH-2**: `WriteTask` 双重序列化（`Vec<u8>` 又反序列化再序列化） | 3 次 serde 操作，性能损耗 | LSP 优化阶段 |
| **HIGH-3**: 离线变更盲区（`git pull` 等检测不到） | 索引与文件系统不同步 | Phase 8 添加 `--force` 重建 |

---

### Phase 7 更新：CLI 命令实现（含方案 A）

基于架构审查，Phase 7 需实现：

#### 1. 修复 CRITICAL-1
- 文件: `projects/cli/src/metadata/index.rs`
- 修改: `index_entity()` 开头添加 `self.remove_entity(&entity.id)?`

#### 2. 实现方案 A: CLI 到 LSP 路由
- **新增**: `projects/cli/src/lsp/client.rs` - CLI 用的 JSON-RPC 客户端
- **修改**: `projects/cli/src/lsp/backend.rs` - 暴露 CLI 方法
- **修改**: `projects/cli/src/commands/metadata.rs` - 检测 LSP 并路由

#### 3. CLI 命令简化实现
```rust
// 伪代码：handle_index
async fn handle_index(cmd: IndexCommand) -> anyhow::Result<()> {
    if let Some(lsp) = LspClient::connect().await? {
        // LSP 运行，路由到 LSP
        let result = lsp.cli_index(cmd.path.to_string()).await?;
        println!("Indexed {} files", result.file_count);
    } else {
        // LSP 未运行，直接操作 sled
        let index = IndexManager::open(cache_dir)?;
        // ... 批量索引 ...
        index.flush()?;
    }
    Ok(())
}
```

---

### 架构审查总结

| 维度 | 评价 | 说明 |
|------|------|------|
| 分层设计 | ✅ 优秀 | Core/CLI 分离清晰 |
| CLI/LSP 分离 | ✅ 正确 | 简化 vs 完整的策略合理 |
| 组件必要性 | ✅ 合理 | CacheManager/WriteBackWorker 都是 LSP 必需的 |
| 并发安全 | ⚠️ 需修复 | CRITICAL-2 需方案 A 实现 |
| 数据一致性 | ⚠️ 需修复 | CRITICAL-1 需 1 行代码修复 |
| 代码复用 | 🟡 可改善 | trait 签名不匹配（HIGH-1） |

**建议**: 先修复 CRITICAL-1（1 行代码），再实现方案 A 的路由机制，然后继续 Phase 7。


---

## 可插拔存储架构设计 (2026-02-27 更新)

### 设计决策

**背景**: sled 在 NFS/CIFS 网络文件系统上存在已知问题（mmap 不可靠、fsync 语义差异、文件锁行为不一致）。

**决策**: 采用**可插拔存储后端**，用户通过配置文件完全自主选择。

### 存储后端选项

| 后端 | 类型 | 适用场景 | 配置示例 |
|------|------|---------|----------|
| **sled** | 嵌入式 KV | 本地 SSD（默认） | `backend = "sled"` |
| **jsondb** | 文本 JSON | NFS/CIFS 环境 | `backend = "jsondb"` |
| **redis** | 远程服务 | 多机共享/集群 | `backend = "redis"` |
| **sqlite** | 嵌入式 SQL | 需要 SQL 查询 | `backend = "sqlite"` |

### 存储位置选项

```toml
[cache]
backend = "sled"  # 或 "jsondb", "redis", "sqlite"

# 位置选项（三选一）
location = "project"    # 项目目录下的 .cache/novelsaga/
location = "home"       # ~/.cache/novelsaga/<project-hash>/
location = "custom"     # 自定义路径
custom_path = "/mnt/fast-ssd/novelsaga-cache/"

# Redis 专用配置（无 location 选项）
[cache.redis]
url = "redis://localhost:6379"
key_prefix = "novelsaga:myproject"
```

### 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                     IndexManager (统一接口)                      │
│  - get_by_id(id)                                                │
│  - list_by_type(type)                                           │
│  - list_by_namespace(ns)                                        │
│  - index_entity(entity)                                         │
│  - remove_entity(id)                                            │
│  - search(query)                                                │
└────────────────────────────┬────────────────────────────────────┘
                             │
              ┌──────────────┼──────────────┬──────────────┐
              ▼              ▼              ▼              ▼
       ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌────────────┐
       │   Sled     │ │   JsonDb   │ │   Redis    │ │   SQLite   │
       │  Backend   │ │  Backend   │ │  Backend   │ │  Backend   │
       ├────────────┤ ├────────────┤ ├────────────┤ ├────────────┤
       │ • 高性能   │ │ • 纯文本   │ │ • 远程共享 │ │ • SQL 支持 │
       │ • 本地 SSD │ │ • NFS 安全 │ │ • 无位置   │ │ • 事务支持 │
       │ • 零拷贝   │ │ • 易调试   │ │   选项     │ │ • 通用     │
       └────────────┘ └────────────┘ └────────────┘ └────────────┘
```

### 配置热更新

**决策**: `notify + debouncer + arc-swap` 方案

**理由**: 
- LSP 服务器需要高频读取配置 → arc-swap 零成本
- 配置不常变更 → notify 事件驱动优于轮询
- 编辑器保存产生多事件 → debounce 必需

**不自动检测文件系统类型**，全部由用户配置决定。

```toml
[watch]
mode = "auto"  # auto/notify/polling
debounce_ms = 300
```

模式说明:
- `auto`: 根据 backend 自动选择（sled→notify, jsondb→polling）
- `notify`: 强制使用 notify（本地文件系统）
- `polling`: 强制使用轮询（NFS/CIFS）

### Phase 9: 可插拔存储实现（新增）

在原有 Phase 7/8 之后增加：

#### 9.1 存储后端 trait 定义

```rust
// projects/core/src/metadata/storage/mod.rs
pub trait StorageBackend: Send + Sync {
    fn get_by_id(&self, id: &str) -> Result<Option<MetadataEntity>>;
    fn list_by_type(&self, type_: &str) -> Result<Vec<MetadataEntity>>;
    fn list_by_namespace(&self, ns: &str) -> Result<Vec<MetadataEntity>>;
    fn index_entity(&self, entity: &MetadataEntity) -> Result<()>;
    fn remove_entity(&self, id: &str) -> Result<()>;
    fn search(&self, query: &str) -> Result<Vec<MetadataEntity>>;
    fn flush(&self) -> Result<()>;
}

pub enum BackendType {
    Sled,
    JsonDb,
    Redis,
    Sqlite,
}
```

#### 9.2 各后端实现

| 后端 | 文件路径 | 说明 |
|------|----------|------|
| sled | `storage/sled.rs` | 现有代码迁移 |
| jsondb | `storage/jsondb.rs` | 单文件 JSON，适合 NFS |
| redis | `storage/redis.rs` | 使用 redis crate |
| sqlite | `storage/sqlite.rs` | 使用 rusqlite |

#### 9.3 工厂模式创建

```rust
// 根据配置创建对应后端
pub fn create_backend(
    backend_type: BackendType,
    location: CacheLocation,
) -> Result<Box<dyn StorageBackend>> {
    match backend_type {
        BackendType::Sled => Ok(Box::new(SledBackend::new(location.path())?)),
        BackendType::JsonDb => Ok(Box::new(JsonDbBackend::new(location.path())?)),
        BackendType::Redis => Ok(Box::new(RedisBackend::new(location.redis_url())?)),
        BackendType::Sqlite => Ok(Box::new(SqliteBackend::new(location.path())?)),
    }
}
```

#### 9.4 配置更新

```toml
# novelsaga.config.toml
[cache]
backend = "sled"
location = "project"  # project/home/custom

# 可选：自定义路径
# custom_path = "/path/to/cache"

[watch]
mode = "auto"      # auto/notify/polling
debounce_ms = 300  # 防抖毫秒
```

### 后端选择指南

| 使用场景 | 推荐后端 | 原因 |
|----------|----------|------|
| 本地开发 | **sled** | 性能最好 |
| NFS/SMB 共享 | **jsondb** | 文本安全，易调试 |
| 多机协作 | **redis** | 共享缓存 |
| 需要 SQL | **sqlite** | 灵活查询 |
| 简单/临时 | **jsondb** | 零依赖，易清理 |

### 实施优先级

1. **Phase 7**: 修复 CRITICAL-1，实现 CLI 命令（sled 单后端）
2. **Phase 8**: 集成测试
3. **Phase 9**: 可插拔存储架构
   - 9.1: 定义 trait + sled 迁移
   - 9.2: jsondb 实现（NFS 支持）
   - 9.3: redis 实现（可选）
   - 9.4: sqlite 实现（可选）

建议先完成 Phase 7/8，让 sled 版本可用，再逐步添加其他后端。