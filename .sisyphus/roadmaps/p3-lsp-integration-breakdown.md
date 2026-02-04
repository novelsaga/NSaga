# P3 - LSP 服务器集成设计概要

## 概述

LSP 服务器功能集成的设计规划，包含文档管理、hover、completion 和诊断功能。

> **注意**: 此为设计概要文档，代码实现细节可能已过时。执行前请验证当前代码结构。

## 功能目标

1. **文档生命周期**: didOpen, didChange, didSave, didClose
2. **Hover**: 悬停显示元数据信息
3. **Completion**: 场景/章节自动补全
4. **Diagnostics**: 损坏链接检测 (可选)

## 现有架构

- LSP backend 骨架: `projects/cli/src/lsp/backend.rs`
- 使用 `tower-lsp` crate
- 异步处理 (`tokio`)

## 实现阶段

| Phase    | 内容               | 任务数 | 预计时间 |
| -------- | ------------------ | ------ | -------- |
| Phase 1  | 文档生命周期管理   | 6      | 1.5 天   |
| Phase 2  | Hover 功能         | 5      | 1.5 天   |
| Phase 3  | Completion 功能    | 6      | 2 天     |
| Phase 4  | 服务器配置和注册   | 3      | 0.5 天   |
| Phase 5  | Diagnostics (可选) | 2      | 1 天     |
| Phase 6  | 集成测试和文档     | 3      | 1 天     |
| Phase 7  | 性能优化 (可选)    | 2      | 0.5 天   |
| **合计** |                    | **28** | **7 天** |

## 关键设计决策

- DocumentStore: `Arc<RwLock<BTreeMap<Url, Document>>>`
- 使用 gray_matter 解析 frontmatter
- Markdown 链接格式: `[text](id:scene-123)`
- 补全触发字符: `[`, `(`

## MVP 范围

完成 Phase 1-4, 6 即可发布核心功能。Phase 5 (诊断) 和 Phase 7 (优化) 可后续添加。

## 依赖关系

**必须先完成**:

- P2 MetadataManager (Phase 4, Task 4.1-4.7)
- P2 State 集成 (Phase 6, Task 6.1-6.3)
