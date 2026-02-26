# P2 - 元数据管理器设计概要

## 概述

小说元数据管理器的设计规划，基于文件系统的元数据管理，使用 Sled 嵌入式 KV 存储作为索引和缓存。

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
        ├── cache.rs                 # moka 内存缓存
        ├── index.rs                 # sled 索引操作
        ├── watcher.rs               # notify 文件监听
        └── worker.rs                # 异步写回 Worker
```

**设计原则**: core 只定义接口和纯逻辑，cli 实现所有 IO 操作。

## 存储架构

- **元数据文件**: `metadata/` 目录下的 Markdown 文件（frontmatter + 正文）
- **缓存/索引**: `.cache/novelsaga/` 下的 Sled 数据库（可重建，可 gitignore）
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

## 缓存策略（cli）

**Write-Back 模式**: 内存同步修改，磁盘异步落盘

- moka 热缓存（LRU，容量可配置）
- 后台 Worker 批量写入文件（flume 通道）
- notify 监听外部变更，失效缓存

详见代码实现（略）。

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
index:name:<id>       # name → path 映射
index:type:<type>     # 类型索引
index:tag:<tag>       # 标签索引
```

## 实现阶段

| Phase    | 模块 | 内容                 | 任务数 | 预计时间    |
| -------- | ---- | -------------------- | ------ | ----------- |
| 1        | core | 数据模型、类型推导器 | 3      | 0.5 天      |
| 2        | core | 查询接口定义         | 2      | 0.25 天     |
| 3        | cli  | moka 缓存层          | 3      | 0.5 天      |
| 4        | cli  | sled 索引层          | 3      | 0.5 天      |
| 5        | cli  | notify 文件监听      | 3      | 0.5 天      |
| 6        | cli  | 异步写回 Worker      | 3      | 0.5 天      |
| 7        | cli  | CLI 命令集成         | 2      | 0.5 天      |
| 8        | -    | 集成测试             | 2      | 0.5 天      |
| **合计** |      |                      | **21** | **3.75 天** |

## 关键设计决策

- **分层**: core 纯逻辑，cli 处理所有 IO
- **异步写回**: 内存同步，磁盘异步（moka + flume + worker）
- **缓存策略**: Write-Back + notify 失效
- **存储分离**: 文件（真相源）+ sled（索引）+ moka（热缓存）

## NovelSaga Workspace 结构要求

### 目录布局（层级不封顶）

约定：任意目录下可放置 `metadata/` 文件夹，该目录即成为一个 scope。scope 可以无限嵌套。

```
workspace/                               # 工作空间（任意名称）
├── .cache/novelsaga/                    # 缓存
│   └── sled/
│
├── novelsaga.config.toml
├── .novelsagaignore
│
├── metadata/                           # 根级 scope
│   └── world-setting.md                # 全局世界观
│
├── book-01/                            # 子目录（一级 scope）
│   ├── metadata/
│   │   └── hero.md                     # 覆盖全局的 hero
│   ├── part-01/
│   │   ├── metadata/
│   │   └── chapter-01/
│   │       ├── metadata/
│   │       │   └── temp-guard.md       # 仅本章可见
│   │       └── scene-01.md
│   └── extra-nested/
│       └── deeper/
│           └── metadata/               # 深层 scope 也有效
│               └── special-event.md
│
└── standalone-story/                   # 另一作品（平级）
    └── metadata/
        └── another-hero.md
```

**核心规则**:
- 任意目录下出现 `metadata/` 即成为一个独立的 scope
- scope 层级无限制（上不封顶）
- 作者自由组织目录结构，无需遵循固定层级名称（book/part/chapter 只是示例）

```
workspace/                               # 工作空间（系列/单作品）
├── .cache/novelsaga/                    # 缓存（可 gitignore）
│   ├── sled/                           # 索引数据库
│   └── ...                             # 其他缓存文件
│
├── novelsaga.config.toml               # 项目配置
├── .novelsagaignore                    # 忽略规则
│
├── metadata/                           # 全局元数据（根级）
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

**就近解析**: 编辑 `book-01/part-01/chapter-01/scene-01.md` 时，查找 `hero` 的顺序：
1. `book-01/part-01/chapter-01/metadata/hero.md`（最近）
2. `book-01/part-01/metadata/hero.md`
3. `book-01/metadata/hero.md`
4. `metadata/hero.md`（最远）

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

## CLI 命令

```bash
novelsaga index              # 重建完整索引
novelsaga index --watch      # 监听模式，增量更新
novelsaga list --type character --namespace book-01
novelsaga show hero          # 显示元数据详情
```

## 依赖

| 模块 | 依赖                | 用途     |
| ---- | ------------------- | -------- |
| core | serde               | 序列化   |
| cli  | moka                | 内存缓存 |
| cli  | sled                | 持久索引 |
| cli  | notify              | 文件监听 |
| cli  | flume/async-channel | 异步通道 |
