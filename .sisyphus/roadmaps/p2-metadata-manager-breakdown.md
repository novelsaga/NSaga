# P2 - 元数据管理器设计概要

## 概述

小说元数据管理器的设计规划，包含数据模型、存储层和查询功能。

> **注意**: 此为设计概要文档，代码实现细节可能已过时。执行前请验证当前代码结构。

## 功能目标

1. **数据模型**: Status、SceneMetadata、ChapterMetadata、NovelMetadata
2. **存储层**: JSON 持久化、YAML 导出
3. **MetadataManager**: CRUD 操作、脏标记优化
4. **查询功能**: 按章节/POV/状态查询场景
5. **集成**: TypeScript 类型导出

## 实现阶段

| Phase    | 内容              | 任务数 | 预计时间   |
| -------- | ----------------- | ------ | ---------- |
| Phase 1  | 基础数据模型      | 6      | 0.5 天     |
| Phase 2  | 错误类型定义      | 2      | 0.25 天    |
| Phase 3  | 存储层实现        | 4      | 0.5 天     |
| Phase 4  | MetadataManager   | 7      | 1.5 天     |
| Phase 5  | 查询和索引 (可选) | 2      | 0.5 天     |
| Phase 6  | State 集成 (可选) | 3      | 0.25 天    |
| Phase 7  | TypeScript 类型   | 1      | 0.1 天     |
| Phase 8  | 集成测试          | 1      | 0.5 天     |
| **合计** |                   | **32** | **3.6 天** |

## 关键设计决策

- 使用 `BTreeMap` (非 HashMap) - 确定性序列化
- 使用 `DateTime<Utc>` - 时间戳标准
- 脏标记 (dirty flag) - 优化保存频率
- 单文件 JSON 存储 (v1)
- `#[derive(TS)]` 导出 TypeScript 类型

## MVP 范围

完成 Phase 1-4, 7-8 即可使用核心功能。Phase 5 (查询) 和 Phase 6 (State 集成) 可后续添加。

## 依赖关系

无外部依赖，可独立开始。
