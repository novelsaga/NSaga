# Stability & Quality - Historical Archive

## TL;DR

NovelSaga 项目关于代码质量、稳定性提升和构建系统优化的历史完成记录存档。

## Context

### Background

NovelSaga 作为一个高性能、多编辑器的小说编辑系统，其核心稳定性和代码质量至关重要。本项目曾经历了一次大规模的代码质量清理，包括消除 Rust Clippy 警告、修复 TypeScript ESLint 错误，以及对构建系统进行优化，以确保开发流程的稳健。

### Current State

这些任务最初记录在 `.opencode/docs/COMPLETED_TASKS.md` 中。为了符合 OpenCode 规范并统一项目计划管理，现将其迁移至 `.sisyphus/plans/` 目录下。

### Constraints

- 必须保留原始的时间戳（2026-01-26 至 2026-02-04）。
- 必须将路径转换为相对项目根目录的路径。
- 所有任务项必须使用 `- [ ]` 格式（OpenCode 规范：计划文件中的所有 checkbox 保持未勾选）。
- 禁止删除或修改原始源文件 `.opencode/docs/COMPLETED_TASKS.md`。

## Objectives

### Primary Goal

保存 NovelSaga 历史稳定性提升和代码清理工作的详细记录，作为后续开发、质量审计和知识传承的参考。

### Success Criteria

- [ ] 完整迁移 `.opencode/docs/COMPLETED_TASKS.md` 中的所有核心完成记录。
- [ ] 保持原有的中文描述、修复内容和技术实现细节。
- [ ] 确保所有文件引用使用正确的相对路径。
- [ ] 迁移后的文档遵循 OpenCode 计划文件标准格式（TL;DR, Context, Objectives, 分类记录）。

## Completed Work Categories

### 1. 代码质量清理 (2026-02-XX)

#### Rust Clippy 警告全面修复

- [ ] **修复目标**: 从 30+ warnings 降至 0 warnings
- [ ] **`projects/cli/src/bridge/manager/mod.rs`**:
  - [ ] `shutdown_all()`: 移除不必要的 `Result<()>` 包装 - 该函数从不失败，只记录错误
  - [ ] `get_workspace_root()`: 将 `map().unwrap_or_else()` 改为 `map_or_else()`
- [ ] **`projects/cli/src/bridge/runtime/discovery.rs`**:
  - [ ] `RuntimeType` enum: 添加 `Copy` derive
  - [ ] `supports_native_typescript()`: 合并相同的 match arms (`Bun | Deno`)
  - [ ] `get_version()`: 移除 `&self` 改为关联函数 `Self::get_version()`
  - [ ] `RuntimeDiscovery` impl: 添加 `#[allow(clippy::unused_self, clippy::trivially_copy_pass_by_ref)]`（ZST 使用实例方法模式）
  - [ ] `RuntimeType` impl: 添加 `#[allow(clippy::trivially_copy_pass_by_ref)]`（1-byte enum，遵循 Rust 惯例）
- [ ] **`projects/cli/src/bridge/runtime/process.rs`**:
  - [ ] `pid()`: 返回类型从 `Option<u32>` 改为 `u32`（始终为 `Some`）
  - [ ] `is_running()`: 合并相同的 match arms: `Ok(Some(_)) | Err(_) => false`
- [ ] **`projects/cli/src/config/loader.rs` (主要重构)**:
  - [ ] 创建 `ConfigLoaderFn` 类型别名解决复杂闭包类型
  - [ ] 创建 `LoaderContext` 结构体将函数参数从 8 个减少到 3 个
  - [ ] 将 `&Option<PathBuf>` 模式改为 `Option<&PathBuf>`
  - [ ] 删除 `load_js_config_impl` 和 `load_ts_config_impl` 中间函数
- [ ] **测试文件 (`projects/cli/src/bridge/rpc/tests.rs`, `projects/cli/src/bridge/runtime/tests.rs`, `projects/cli/src/bridge/transport/tests.rs`)**:
  - [ ] doc comments 中的代码标识符添加反引号 (`doc_markdown` lint)
  - [ ] 内联 format 字符串变量 (`{e}` 替代 `{}`, e)
  - [ ] `if let Ok(_) = x` 改为 `x.is_ok()`

#### TypeScript ESLint 错误全面修复

- [ ] **修复目标**: 从 42 errors 降至 0 errors
- [ ] `projects/cli-js-bridges/bridge-core/src/interfaces/service.ts`: 修复 `Promise<unknown> | unknown` 冗余联合类型 → `unknown`
- [ ] `projects/cli-js-bridges/bridge-core/src/rpc-handler.ts`: 正确类型标注，`!` 替换为 `?? null`
- [ ] `projects/cli-js-bridges/bridge-deno/src/index.ts`: 使用 `instanceof Error` 检查修复 unsafe `any` 参数
- [ ] `projects/cli-js-bridges/bridge-deno/src/transport.ts`: 添加 eslint-disable for triple-slash reference
- [ ] `projects/cli-js-bridges/bridge-nodejs/src/transport.ts`: 为 floating promises 添加 `void`
- [ ] `projects/cli-js-bridges/bridge-nodejs/build.mts`: 重写格式化并为 floating promise 添加 `void`
- [ ] `projects/cli-js-bridges/bridge-bun/src/transport.ts`: 为 floating promises 添加 `void`
- [ ] `projects/cli-js-bridges/config-bridge/src/services/config.ts`: 修复冗余类型联合，正确类型标注
- [ ] `projects/cli-js-bridges/config-bridge/src/index.ts`: catch 参数改为 `unknown` 类型

#### E2E 测试验证 (2026-02-XX)

- [ ] **验证状态**: 14 个测试场景全部通过
- [ ] 测试涵盖: Config Priority, Error Handling, Node.js + JS/TS (ESM/CJS), Bun + JS/TS (ESM/CJS), Deno + JS/TS (ESM/CJS)

### 2. 构建系统与配置 (2026-02-04)

#### 配置文件修复和构建系统优化

- [ ] 将生成的 TS 类型文件路径从旧位置 `projects/cli/assets/js/src/js-config-bridge/src/types` 更改为新位置 `projects/cli-js-bridges/config-bridge/src/types`
- [ ] 优化 TypeScript 配置服务，使用私有字段语法 `#settings` 提高封装性
- [ ] 添加 `projects/cli-js-bridges/config-bridge/src/types/.gitignore` 以忽略自动生成的 `_config.ts` 文件
- [ ] 优化 xtask 构建流程，将类型生成整合到构建步骤中，确保顺序执行
- [ ] 更新 OpenCode LSP 配置以支持 Rust、TypeScript 和 Nix 的语言服务器
- [ ] 修复 ESLint 配置，允许对自动生成文件使用无限制的 eslint-disable 注释
- [ ] 更新 Nix 配置文件，修正依赖哈希值和格式化问题
- [ ] **文件修改清单**:
  - [ ] `.opencode/config.json` - 添加 LSP 配置支持
  - [ ] `eslint.config.mts` - 调整规则以适应生成文件
  - [ ] `packages.nix` - 更新依赖哈希和格式化
  - [ ] `projects/cli-js-bridges/config-bridge/src/services/config.ts` - 优化服务实现
  - [ ] `projects/cli-js-bridges/config-bridge/src/types/.gitignore` - 新增忽略规则
  - [ ] `xtask/src/tasks/bindings.rs` - 更新生成路径
  - [ ] `xtask/src/tasks/build.rs` - 整合类型生成到构建流程

#### ESLint 和 Prettier 配置优化 (2026-02-02)

- [ ] **问题 1: ESLint 忽略根目录配置文件**
  - [ ] 修复 `eslint.config.mts` 中 `commonTsFile` 的匹配逻辑，确保根目录文件如 `eslint.config.mts` 被正确识别。
- [ ] **问题 2: Prettier 配置改进**
  - [ ] 移除手动 YAML 解析，改用官方 pnpm API (`@pnpm/find-workspace-dir`, `@pnpm/lockfile-file`)。
  - [ ] 提升了类型安全性、格式兼容性和官方维护支持。
- [ ] **问题 3: TypeScript 导入规范规则**
  - [ ] 添加 `@typescript-eslint/consistent-type-imports` 规则，强制 type-only imports 使用 `type` 关键字。
- [ ] **依赖包清单完善**:
  - [ ] 确认已安装并配置: `@eslint/js`, `typescript-eslint`, `eslint-config-prettier`, `eslint-plugin-prettier` 等。

### 3. CLI 运行时功能 (2026-01-26)

#### 清理编译警告 (1.1)

- [ ] 常量复用：重构硬编码为常量引用。
- [ ] E2E 测试扩展：实现运行时矩阵测试。
- [ ] 架构优化：重构 CLI `ConfigLoader`，使其专职于 JS/TS 动态加载。

#### 集成用户指定的运行时路径 (1.1.1)

- [ ] 支持通过 `--runtime` 选择运行时（auto, node, bun, deno）。
- [ ] 支持通过 `--node-path`, `--bun-path`, `--deno-path` 指定具体执行文件路径。
- [ ] `ConfigLoader` 现已接受 CLI 参数并正确传递给 `RuntimeDiscovery`。
- [ ] **涉及文件**: `projects/cli/src/bridge/runtime/discovery.rs`, `projects/cli/src/args/mod.rs`, `projects/cli/src/config/loader.rs`

#### JS Bridge 自动构建集成 (1.1.2)

- [ ] 实现 `xtask build-js` 命令，支持 `--force` 强制重建。
- [ ] 实现增量构建逻辑：基于 mtime 对比，自动跳过未修改的 bridge。
- [ ] 智能过滤：跳过 `node_modules` 和 `dist` 目录。
- [ ] 集成至 `xtask build-all` 和 E2E 测试流程。
- [ ] **涉及文件**: `xtask/src/tasks/build.rs`

#### 清理废弃的测试文件 (1.1.4)

- [ ] 删除 `projects/cli/assets/test/` 目录。
- [ ] 将所有依赖该目录的单元测试（共 9 处）重构为使用 `tempfile` 动态生成配置。
- [ ] 实现了测试环境的纯净和动态化。

#### 静态配置格式支持 (1.3)

- [ ] 架构确认：Core 的 `ConfigManager` 负责 .toml/.json/.yaml 静态格式发现和加载。
- [ ] CLI 的 `ConfigLoader` 仅提供动态脚本加载闭包。
- [ ] 明确了 Core 与 CLI 在配置处理上的职责边界。

### 4. 性能优化 (部分)

#### 配置缓存实现 (3.2)

- [ ] Core 的 `ConfigManager` 已实现基于 mtime 的配置缓存逻辑。
- [ ] 支持配置热重载：检测到文件修改后通过 `del_override_config_cache()` 清除缓存。
- [ ] 使用 `Arc<RwLock<HashMap<...>>>` 确保线程安全。
- [ ] **涉及文件**: `projects/core/src/state/manager/config.rs`
