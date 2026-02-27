# NovelSaga CLI 开发路线图

## 概述

NovelSaga CLI 后续开发的优先级路线图。

## 优先级列表

### P1 - 立即开始 ✅

- [x] **错误消息优化** - 改进用户友好的错误提示
  - 完成: 2026-02-04, commit 8fa7950

### P2 - 接下来 ✅ (已完成)

- [x] **元数据管理器** - 小说元数据管理（core 定义接口，cli 实现 IO）
  - ✅ **core 层完成** (2026-02)
    - `MetadataEntity` 数据模型
    - 类型推导、路径解析
    - 查询接口 `MetadataQuery` trait
  - ✅ **cli 基础层完成** (2026-02)
    - `IndexManager` (sled) - **CLI+LSP 共用**
  - ✅ **cli LSP 层完成** (2026-02)
    - moka 内存缓存 (`CacheManager`) - **仅 LSP**
    - notify 文件监听 (`FileWatcher`) - **仅 LSP (可选)**
    - 异步写回 Worker (`WriteBackWorker`) - **仅 LSP**
  - ✅ **已完成** (2026-02-27)
    - `novelsaga index` - 直接扫描（已实现）
    - `novelsaga list` - 查询 sled（已实现）
    - `novelsaga show` - 显示详情（已实现）
  - **架构说明**: 
    - CLI 命令使用**简化架构**：直接读写 `IndexManager`
    - LSP 使用**完整架构**：`CacheManager` + `WriteBackWorker` + 可选 `FileWatcher`
  - **详细设计**: 见 `p2-metadata-manager-breakdown.md`

### P3 - 接下来 ⏳ (准备开始)

- [ ] **LSP 服务器集成** - hover, completion 功能
  - **详细设计**: 见 `p3-lsp-integration-breakdown.md`
  - **可用组件**: `IndexManager`, `CacheManager`, `WriteBackWorker` (P2 已完成)
  - **监听策略**: 优先使用 LSP hooks (didOpen/didChange/didSave)，`FileWatcher` 降级备用
  - ✅ **当前状态**: P2 已完成，阻塞解除

## 优先级矩阵

| 优先级 | 任务           | 状态      | 预计工作量 | 备注 |
| ------ | -------------- | --------- | ---------- | ---- |
| P1     | 错误消息优化   | ✅ 完成   | 1-2 天     | 2026-02-04 |
| P2     | 元数据管理器   | ✅ 完成   | 3-4 天     | 2026-02-27 |
|        | - Core         | ✅ 完成   |            |            |
|        | - IndexManager | ✅ 完成   |            | CLI+LSP 共用 |
|        | - LSP 组件     | ✅ 完成   |            | Cache/Worker/Watcher |
|        | - CLI 命令     | ✅ 完成   |            | index/list/show |
| P3     | LSP 服务器集成 | ⏳ 准备开始 | 5-7 天   | 依赖已解除，组件就绪 |

## 暂缓任务

见 `future-enhancements.md`:

- 稳健性: RPC 超时、Bridge 崩溃日志收集
- 体验: 配置热重载
- 性能: Bridge 预热 (Warmup)
- 运维: CI/CD 流水线


## 架构审查更新 (2026-02-27)

### 审查结论

- ✅ **整体架构合理**：Core/CLI 分层清晰，组件选型成熟
- ⚠️ **发现 2 个 CRITICAL 问题**：需在 P2 Phase 7 前处理
- ✅ **用户决策**：采用方案 A（CLI 路由到 LSP）

### CRITICAL 问题与方案

#### 🔴 CRITICAL-1: `index_entity()` 旧索引残留
- **问题**：entity type/namespace 变更后，旧二级索引不清除
- **影响**：`list_by_type()` 返回幽灵数据
- **修复**：1 行代码（`self.remove_entity(&entity.id)?`）
- **详细设计**：见 `p2-metadata-manager-breakdown.md` 架构审查章节

#### 🔴 CRITICAL-2: sled 多进程访问冲突
- **问题**：LSP 运行时 CLI 命令无法同时打开 sled
- **方案 A（已采纳）**：CLI 检测 LSP，通过 JSON-RPC 路由命令
- **涉及文件**：
  - 新增：`projects/cli/src/lsp/client.rs`（JSON-RPC 客户端）
  - 修改：`projects/cli/src/lsp/backend.rs`（暴露 CLI 方法）
  - 修改：`projects/cli/src/commands/metadata.rs`（路由逻辑）
- **详细设计**：见 `p2-metadata-manager-breakdown.md` 架构审查章节

### P2 Phase 7 更新任务

基于架构审查，Phase 7 现在包含：

| 子任务 | 优先级 | 说明 |
|--------|--------|------|
| 修复 CRITICAL-1 | 🔴 P0 | `index_entity()` 添加 `remove_entity()` 调用 |
| 实现 LSP Client | 🔴 P0 | CLI 用的 JSON-RPC 客户端 |
| 暴露 LSP CLI 方法 | 🔴 P0 | backend.rs 添加 `cli_index/list/show` |
| 实现路由逻辑 | 🔴 P0 | commands/metadata.rs 检测 LSP 并路由 |
| CLI 命令简化实现 | 🟡 P1 | index/list/show 使用上述机制 |
| 集成测试 | 🟡 P1 | 测试 CLI 直接模式和 LSP 路由模式 |

### 工作量重新评估

原预计 Phase 7: 0.5 天 → **更新后: 1-1.5 天**（含方案 A 实现）

方案 A 增加了 JSON-RPC 客户端/服务器实现，但避免了长期的 sled 并发问题。
