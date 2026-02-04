# Plan: P2 - 元数据管理器 (Metadata Manager) - 详细任务拆分

## TL;DR

将 P2 元数据管理器任务拆分为细粒度的子任务，每个任务对应一个文件或模块的创建/修改。

## Context

- **父任务**: `.sisyphus/plans/cli-next-steps.md` - P2 元数据管理器
- **预计总工作量**: 3-5 天
- **拆分原则**: 一个文件一个任务，按依赖关系排序
- **可并行性**: 标注哪些任务可以并行执行

## Research Summary

基于 exploration-findings.md 和 librarian-research.md 的研究结果：

**内部模式** (从现有代码):

- 使用 `#[derive(TS)]` + `#[ts(export, export_to = "_metadata.ts")]` 导出类型
- ConfigManager 缓存模式: `Arc<RwLock<BTreeMap<K, V>>>`
- StateBuilder + Initializer 集成模式

**外部最佳实践**:

- BTreeMap (非 HashMap) - 确定性序列化
- DateTime<Utc> - 时间戳标准
- 脏标记 (dirty flag) - 优化保存
- 单文件 JSON 存储 (v1)
- 内存二级索引

## TODO List

### Phase 1: 基础数据模型 (可并行)

- [ ] **Task 1.1: 创建 metadata 模块入口**
  - **文件**: `projects/core/src/metadata/mod.rs`
  - **内容**: 模块声明、公共导出
  - **依赖**: 无
  - **可并行**: ✅ 是 (独立任务)
  - **验证**: `cargo build -p novelsaga-core` 通过

- [ ] **Task 1.2: 定义 Status 枚举**
  - **文件**: `projects/core/src/metadata/model.rs` (新建)
  - **内容**:
    - `Status` 枚举 (Draft, Review, Revised, Final)
    - serde + TS 导出属性
    - Default 实现
  - **依赖**: Task 1.1 完成
  - **可并行**: ✅ 是 (与 Task 1.3-1.5 并行)
  - **验证**: `cargo test -p novelsaga-core test_status_serialization`

- [ ] **Task 1.3: 定义 SceneMetadata 结构体**
  - **文件**: `projects/core/src/metadata/model.rs` (追加)
  - **内容**:
    - `SceneMetadata` 结构体 (id, title, chapter, position, pov, synopsis, status, word_count, custom_fields, created_at, updated_at, include_in_compile)
    - 使用 `BTreeMap<String, String>` 作为 custom_fields
    - 使用 `chrono::DateTime<Utc>` 作为时间戳
    - serde 属性: `#[serde(default)]`, `#[serde(skip_serializing_if = "...")]`
    - TS 导出: `#[ts(export, export_to = "_metadata.ts")]`
    - Default 实现
  - **依赖**: Task 1.2 完成 (需要 Status 枚举)
  - **可并行**: ❌ 否 (依赖 Status)
  - **验证**: `cargo test -p novelsaga-core test_scene_metadata_serialization`

- [ ] **Task 1.4: 定义 ChapterMetadata 结构体**
  - **文件**: `projects/core/src/metadata/model.rs` (追加)
  - **内容**:
    - `ChapterMetadata` 结构体 (id, title, number, synopsis, scenes, word_count, created_at, updated_at)
    - 使用 `Vec<String>` 存储 scene IDs
    - serde + TS 导出
    - Default 实现
  - **依赖**: Task 1.1 完成
  - **可并行**: ✅ 是 (与 Task 1.2-1.3 并行)
  - **验证**: `cargo test -p novelsaga-core test_chapter_metadata_serialization`

- [ ] **Task 1.5: 定义 NovelMetadata 结构体**
  - **文件**: `projects/core/src/metadata/model.rs` (追加)
  - **内容**:
    - `NovelMetadata` 结构体 (id, title, author, synopsis, chapters, total_word_count, created_at, updated_at)
    - 使用 `Vec<String>` 存储 chapter IDs
    - serde + TS 导出
    - Default 实现
  - **依赖**: Task 1.1 完成
  - **可并行**: ✅ 是 (与 Task 1.2-1.4 并行)
  - **验证**: `cargo test -p novelsaga-core test_novel_metadata_serialization`

- [ ] **Task 1.6: 更新 metadata/mod.rs 导出数据模型**
  - **文件**: `projects/core/src/metadata/mod.rs` (修改)
  - **内容**: 导出 Status, SceneMetadata, ChapterMetadata, NovelMetadata
  - **依赖**: Task 1.2-1.5 全部完成
  - **可并行**: ❌ 否 (依赖所有数据模型)
  - **验证**: `cargo build -p novelsaga-core` 通过

### Phase 2: 错误类型定义 (独立)

- [ ] **Task 2.1: 定义 MetadataError 错误类型**
  - **文件**: `projects/core/src/metadata/error.rs` (新建)
  - **内容**:
    - `MetadataError` 枚举 (NotFound, Io, Json, Yaml, InvalidId, DuplicateId)
    - 使用 `thiserror::Error` derive
    - 定义 `type Result<T> = std::result::Result<T, MetadataError>`
  - **依赖**: Task 1.1 完成
  - **可并行**: ✅ 是 (与 Phase 1 并行)
  - **验证**: `cargo test -p novelsaga-core test_metadata_error_display`

- [ ] **Task 2.2: 更新 metadata/mod.rs 导出错误类型**
  - **文件**: `projects/core/src/metadata/mod.rs` (修改)
  - **内容**: 导出 MetadataError, Result
  - **依赖**: Task 2.1 完成
  - **可并行**: ❌ 否 (依赖 Task 2.1)
  - **验证**: `cargo build -p novelsaga-core` 通过

### Phase 3: 存储层实现

- [ ] **Task 3.1: 实现 JSON 保存功能**
  - **文件**: `projects/core/src/metadata/storage.rs` (新建)
  - **内容**:
    - `save_json<T: Serialize>(path: &Path, data: &T) -> Result<()>`
    - 自动创建父目录
    - 使用 `serde_json::to_string_pretty`
    - 原子写入 (写临时文件 → 重命名)
  - **依赖**: Task 1.6, Task 2.2 完成
  - **可并行**: ✅ 是 (与 Task 3.2-3.3 并行)
  - **验证**: `cargo test -p novelsaga-core test_save_json`

- [ ] **Task 3.2: 实现 JSON 加载功能**
  - **文件**: `projects/core/src/metadata/storage.rs` (追加)
  - **内容**:
    - `load_json<T: DeserializeOwned>(path: &Path) -> Result<T>`
    - 处理文件不存在的情况
    - 使用 `serde_json::from_str`
  - **依赖**: Task 1.6, Task 2.2 完成
  - **可并行**: ✅ 是 (与 Task 3.1, 3.3 并行)
  - **验证**: `cargo test -p novelsaga-core test_load_json`

- [ ] **Task 3.3: 实现 YAML 导出功能**
  - **文件**: `projects/core/src/metadata/storage.rs` (追加)
  - **内容**:
    - `export_yaml<T: Serialize>(path: &Path, data: &T) -> Result<()>`
    - 使用 `serde_yaml::to_writer`
  - **依赖**: Task 1.6, Task 2.2 完成
  - **可并行**: ✅ 是 (与 Task 3.1-3.2 并行)
  - **验证**: `cargo test -p novelsaga-core test_export_yaml`

- [ ] **Task 3.4: 更新 metadata/mod.rs 导出存储函数**
  - **文件**: `projects/core/src/metadata/mod.rs` (修改)
  - **内容**: 导出 storage 模块的公共函数
  - **依赖**: Task 3.1-3.3 全部完成
  - **可并行**: ❌ 否 (依赖所有存储函数)
  - **验证**: `cargo build -p novelsaga-core` 通过

### Phase 4: MetadataManager 实现

- [ ] **Task 4.1: 定义 MetadataManager 结构体**
  - **文件**: `projects/core/src/metadata/manager.rs` (新建)
  - **内容**:
    - `MetadataManager` 结构体
    - 字段: `scenes: Arc<RwLock<BTreeMap<String, SceneMetadata>>>`
    - 字段: `chapters: Arc<RwLock<BTreeMap<String, ChapterMetadata>>>`
    - 字段: `novel: Arc<RwLock<Option<NovelMetadata>>>`
    - 字段: `dirty: Arc<RwLock<bool>>`
    - 字段: `storage_path: Option<PathBuf>`
  - **依赖**: Task 1.6, Task 2.2 完成
  - **可并行**: ❌ 否 (依赖数据模型和错误类型)
  - **验证**: `cargo build -p novelsaga-core` 通过

- [ ] **Task 4.2: 实现 MetadataManager::new()**
  - **文件**: `projects/core/src/metadata/manager.rs` (追加)
  - **内容**:
    - `new() -> Self` 构造函数
    - 初始化空的 BTreeMap
  - **依赖**: Task 4.1 完成
  - **可并行**: ❌ 否 (依赖 Task 4.1)
  - **验证**: `cargo test -p novelsaga-core test_metadata_manager_new`

- [ ] **Task 4.3: 实现 Scene CRUD 方法**
  - **文件**: `projects/core/src/metadata/manager.rs` (追加)
  - **内容**:
    - `create_scene(&mut self, metadata: SceneMetadata) -> Result<()>`
    - `get_scene(&self, id: &str) -> Result<SceneMetadata>`
    - `update_scene(&mut self, id: &str, metadata: SceneMetadata) -> Result<()>`
    - `delete_scene(&mut self, id: &str) -> Result<()>`
    - 每次修改设置 `dirty = true`
  - **依赖**: Task 4.2 完成
  - **可并行**: ✅ 是 (与 Task 4.4-4.5 并行)
  - **验证**: `cargo test -p novelsaga-core test_scene_crud`

- [ ] **Task 4.4: 实现 Chapter CRUD 方法**
  - **文件**: `projects/core/src/metadata/manager.rs` (追加)
  - **内容**:
    - `create_chapter(&mut self, metadata: ChapterMetadata) -> Result<()>`
    - `get_chapter(&self, id: &str) -> Result<ChapterMetadata>`
    - `update_chapter(&mut self, id: &str, metadata: ChapterMetadata) -> Result<()>`
    - `delete_chapter(&mut self, id: &str) -> Result<()>`
    - 每次修改设置 `dirty = true`
  - **依赖**: Task 4.2 完成
  - **可并行**: ✅ 是 (与 Task 4.3, 4.5 并行)
  - **验证**: `cargo test -p novelsaga-core test_chapter_crud`

- [ ] **Task 4.5: 实现 Novel CRUD 方法**
  - **文件**: `projects/core/src/metadata/manager.rs` (追加)
  - **内容**:
    - `set_novel(&mut self, metadata: NovelMetadata) -> Result<()>`
    - `get_novel(&self) -> Result<NovelMetadata>`
    - 每次修改设置 `dirty = true`
  - **依赖**: Task 4.2 完成
  - **可并行**: ✅ 是 (与 Task 4.3-4.4 并行)
  - **验证**: `cargo test -p novelsaga-core test_novel_crud`

- [ ] **Task 4.6: 实现持久化方法**
  - **文件**: `projects/core/src/metadata/manager.rs` (追加)
  - **内容**:
    - `set_storage_path(&mut self, path: PathBuf)`
    - `save(&mut self) -> Result<()>` - 保存到 JSON
    - `load(&mut self, path: &Path) -> Result<()>` - 从 JSON 加载
    - `save_if_dirty(&mut self) -> Result<()>` - 仅在 dirty 时保存
    - `export_yaml(&self, path: &Path) -> Result<()>` - 导出 YAML
  - **依赖**: Task 4.3-4.5, Task 3.4 完成
  - **可并行**: ❌ 否 (依赖 CRUD 和存储函数)
  - **验证**: `cargo test -p novelsaga-core test_manager_persistence`

- [ ] **Task 4.7: 更新 metadata/mod.rs 导出 MetadataManager**
  - **文件**: `projects/core/src/metadata/mod.rs` (修改)
  - **内容**: 导出 MetadataManager
  - **依赖**: Task 4.6 完成
  - **可并行**: ❌ 否 (依赖 MetadataManager 完整实现)
  - **验证**: `cargo build -p novelsaga-core` 通过

### Phase 5: 查询和索引 (可选增强)

- [ ] **Task 5.1: 实现基础查询方法**
  - **文件**: `projects/core/src/metadata/manager.rs` (追加)
  - **内容**:
    - `list_scenes(&self) -> Vec<SceneMetadata>`
    - `list_chapters(&self) -> Vec<ChapterMetadata>`
    - `find_scenes_by_chapter(&self, chapter_id: &str) -> Vec<SceneMetadata>`
  - **依赖**: Task 4.7 完成
  - **可并行**: ❌ 否 (依赖 MetadataManager)
  - **验证**: `cargo test -p novelsaga-core test_basic_queries`

- [ ] **Task 5.2: 实现高级查询方法**
  - **文件**: `projects/core/src/metadata/manager.rs` (追加)
  - **内容**:
    - `find_scenes_by_pov(&self, pov: &str) -> Vec<SceneMetadata>`
    - `find_scenes_by_status(&self, status: Status) -> Vec<SceneMetadata>`
    - `get_total_word_count(&self) -> usize`
  - **依赖**: Task 5.1 完成
  - **可并行**: ❌ 否 (依赖基础查询)
  - **验证**: `cargo test -p novelsaga-core test_advanced_queries`

### Phase 6: 集成到 Core State (可选)

- [ ] **Task 6.1: 更新 projects/core/src/lib.rs**
  - **文件**: `projects/core/src/lib.rs` (修改)
  - **内容**: 添加 `pub mod metadata;` 声明
  - **依赖**: Task 4.7 完成 (metadata 模块完整)
  - **可并行**: ❌ 否 (依赖 metadata 模块)
  - **验证**: `cargo build -p novelsaga-core` 通过

- [ ] **Task 6.2: 添加 metadata_manager 到 State**
  - **文件**: `projects/core/src/state/_state.rs` (修改)
  - **内容**:
    - 添加字段 `metadata_manager: MetadataManager` 到 State
    - 添加 getter: `pub fn metadata_manager(&self) -> &MetadataManager`
  - **依赖**: Task 6.1 完成
  - **可并行**: ❌ 否 (依赖 Task 6.1)
  - **验证**: `cargo build -p novelsaga-core` 通过

- [ ] **Task 6.3: 更新 StateBuilder**
  - **文件**: `projects/core/src/state/_state.rs` (修改)
  - **内容**:
    - 添加 `metadata_manager` 字段到 StateBuilder
    - 在 `build()` 中初始化 MetadataManager
  - **依赖**: Task 6.2 完成
  - **可并行**: ❌ 否 (依赖 Task 6.2)
  - **验证**: `cargo test -p novelsaga-core` 通过

### Phase 7: TypeScript 类型生成

- [ ] **Task 7.1: 生成 TypeScript 类型**
  - **文件**: `projects/cli-js-bridges/config-bridge/src/types/_metadata.ts` (自动生成)
  - **内容**: 运行 `cargo test -p novelsaga-core export_bindings`
  - **依赖**: Task 1.6 完成 (所有数据模型定义完成)
  - **可并行**: ❌ 否 (依赖所有带 TS 导出的类型)
  - **验证**: 检查生成的 `_metadata.ts` 文件存在且有效

### Phase 8: 集成测试

- [ ] **Task 8.1: 编写端到端测试**
  - **文件**: `projects/core/src/metadata/manager.rs` (追加 #[cfg(test)] mod)
  - **内容**:
    - 完整场景测试: 创建 novel → 添加 chapters → 添加 scenes → 保存 → 加载 → 验证
    - 使用 tempfile 创建临时测试目录
  - **依赖**: Task 4.7 完成
  - **可并行**: ❌ 否 (依赖完整实现)
  - **验证**: `cargo test -p novelsaga-core test_metadata_manager_e2e`

## Parallelization Map

**可并行的任务组**:

**Group 1 (Phase 1 数据模型)**:

- Task 1.2 (Status)
- Task 1.4 (ChapterMetadata)
- Task 1.5 (NovelMetadata)
  这三个可以同时进行，然后 Task 1.3 (SceneMetadata) 依赖 Task 1.2 完成后执行。

**Group 2 (Phase 2 错误类型 + Phase 1)**:

- Task 2.1 (MetadataError) 可以与 Phase 1 的所有任务并行

**Group 3 (Phase 3 存储层)**:

- Task 3.1 (save_json)
- Task 3.2 (load_json)
- Task 3.3 (export_yaml)
  这三个可以同时进行。

**Group 4 (Phase 4 CRUD)**:

- Task 4.3 (Scene CRUD)
- Task 4.4 (Chapter CRUD)
- Task 4.5 (Novel CRUD)
  这三个可以同时进行。

## Estimated Timeline

| Phase     | Tasks        | Estimated Time | Dependencies   |
| --------- | ------------ | -------------- | -------------- |
| Phase 1   | Task 1.1-1.6 | 0.5 天         | 无             |
| Phase 2   | Task 2.1-2.2 | 0.25 天        | Phase 1        |
| Phase 3   | Task 3.1-3.4 | 0.5 天         | Phase 1, 2     |
| Phase 4   | Task 4.1-4.7 | 1.5 天         | Phase 1, 2, 3  |
| Phase 5   | Task 5.1-5.2 | 0.5 天         | Phase 4 (可选) |
| Phase 6   | Task 6.1-6.3 | 0.25 天        | Phase 4 (可选) |
| Phase 7   | Task 7.1     | 0.1 天         | Phase 1        |
| Phase 8   | Task 8.1     | 0.5 天         | Phase 4        |
| **Total** | **32 tasks** | **3.6 天**     |                |

## Verification Checklist

每个任务完成后必须验证:

- [ ] `cargo build -p novelsaga-core` 通过
- [ ] `cargo test -p novelsaga-core` 全部通过
- [ ] `cargo clippy --all-targets` 0 warnings
- [ ] `lsp_diagnostics` 0 errors
- [ ] 任务在 plan 中标记为完成

## Notes

- **MVP**: 完成 Phase 1-4, 7-8 即可使用 (核心功能)
- **Enhancement**: Phase 5 (查询) 和 Phase 6 (State 集成) 可后续添加
- **TS 类型**: Phase 7 应在 Phase 1 完成后立即执行，确保 JS bridge 可用
- **测试驱动**: 每个 task 都应包含对应的单元测试
