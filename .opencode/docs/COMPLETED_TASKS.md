# NovelSaga Completed Tasks

> 此文档记录了 NovelSaga 项目开发过程中已完成的任务历史。

## 历史完成记录

### 代码质量清理 (2026-02-XX)

#### ~~任务: Rust Clippy 警告全面修复~~ ✅ 已完成

**修复内容**: 从 30+ warnings 降至 0 warnings

##### 文件修改清单

**`projects/cli/src/bridge/manager/mod.rs`**

- `shutdown_all()`: 移除不必要的 `Result<()>` 包装 - 该函数从不失败，只记录错误
- `get_workspace_root()`: 将 `map().unwrap_or_else()` 改为 `map_or_else()`

**`projects/cli/src/bridge/runtime/discovery.rs`**

- `RuntimeType` enum: 添加 `Copy` derive
- `supports_native_typescript()`: 合并相同的 match arms (`Bun | Deno`)
- `get_version()`: 移除 `&self` 改为关联函数 `Self::get_version()`
- `RuntimeDiscovery` impl: 添加 `#[allow(clippy::unused_self, clippy::trivially_copy_pass_by_ref)]`（ZST 使用实例方法模式）
- `RuntimeType` impl: 添加 `#[allow(clippy::trivially_copy_pass_by_ref)]`（1-byte enum，遵循 Rust 惯例）

**`projects/cli/src/bridge/runtime/process.rs`**

- `pid()`: 返回类型从 `Option<u32>` 改为 `u32`（始终为 `Some`）
- `is_running()`: 合并相同的 match arms: `Ok(Some(_)) | Err(_) => false`

**`projects/cli/src/config/loader.rs`** (主要重构)

- 创建 `ConfigLoaderFn` 类型别名解决复杂闭包类型
- 创建 `LoaderContext` 结构体将函数参数从 8 个减少到 3 个
- 将 `&Option<PathBuf>` 模式改为 `Option<&PathBuf>`
- 删除 `load_js_config_impl` 和 `load_ts_config_impl` 中间函数

**测试文件** (`rpc/tests.rs`, `runtime/tests.rs`, `transport/tests.rs`)

- doc comments 中的代码标识符添加反引号 (`doc_markdown` lint)
- 内联 format 字符串变量 (`{e}` 替代 `{}`, e)
- `if let Ok(_) = x` 改为 `x.is_ok()`

---

#### ~~任务: TypeScript ESLint 错误全面修复~~ ✅ 已完成

**修复内容**: 从 42 errors 降至 0 errors

##### 文件修改清单

- `bridge-core/src/interfaces/service.ts`: 修复 `Promise<unknown> | unknown` 冗余联合类型 → `unknown`
- `bridge-core/src/rpc-handler.ts`: 正确类型标注，`!` 替换为 `?? null`
- `bridge-deno/src/index.ts`: 使用 `instanceof Error` 检查修复 unsafe `any` 参数
- `bridge-deno/src/transport.ts`: 添加 eslint-disable for triple-slash reference
- `bridge-nodejs/src/transport.ts`: 为 floating promises 添加 `void`
- `bridge-nodejs/build.mts`: 重写格式化并为 floating promise 添加 `void`
- `bridge-bun/src/transport.ts`: 为 floating promises 添加 `void`
- `config-bridge/src/services/config.ts`: 修复冗余类型联合，正确类型标注
- `config-bridge/src/index.ts`: catch 参数改为 `unknown` 类型

---

#### ~~任务: E2E 测试验证~~ ✅ 全部通过

**14 个测试场景全部通过**:

- Config Priority ✅
- Error Handling ✅
- Node.js + JS ESM/CJS (.mjs/.cjs) ✅
- Node.js + TS ESM/CJS (.mts/.cts) ✅
- Bun + JS ESM/CJS (.mjs/.cjs) ✅
- Bun + TS ESM/CJS (.mts/.cts) ✅
- Deno + JS ESM/CJS (.mjs/.cjs) ✅
- Deno + TS ESM/CJS (.mts/.cts) ✅

---

### 优先级 P1: 代码质量和稳定性

#### ~~任务 1.1: 清理编译警告~~ ✅ 已完成 (2026-01-26)

- ✅ 常量复用：重构硬编码为常量引用
- ✅ E2E 测试扩展：运行时矩阵测试
- ✅ 架构优化：CLI ConfigLoader 只负责 JS/TS 加载（重构完成）
- 📝 剩余：决定是否删除未使用的方法或标记为 `#[allow(dead_code)]`

---

#### ~~任务 1.1.1: 集成用户指定的运行时路径~~ ✅ 已完成 (2026-01-26)

**文件**: `projects/cli/src/bridge/runtime/discovery.rs`, `projects/cli/src/args/mod.rs`, `projects/cli/src/config/loader.rs`

**实现内容**:

- ✅ `ConfigLoader` 接受 CLI 参数（运行时选择和路径）
- ✅ `RuntimeDiscovery::find_runtime_with_preference()` 使用用户指定路径
- ✅ `main.rs` 传递 CLI 参数给 `ConfigLoader`
- ✅ 支持 `--runtime` 选择运行时类型（auto, node, bun, deno）
- ✅ 支持 `--node-path`, `--bun-path`, `--deno-path` 指定路径
- ✅ 消除所有编译警告

---

#### ~~任务 1.1.2: JS Bridge 自动构建集成~~ ✅ 已完成 (2026-01-26)

**文件**: `xtask/src/tasks/build.rs`

**实现内容**:

- ✅ 创建 `xtask build-js` 命令（支持 `--force` 强制重建）
- ✅ 自动检测 JS bridges 需要构建（基于 mtime 对比）
- ✅ 递归检查源文件修改时间，智能跳过未修改的 bridge
- ✅ 支持 `xtask clean-js` 清理构建产物
- ✅ 集成到 `xtask build-all` 和 E2E 测试流程
- ✅ 跳过 `node_modules` 和 `dist` 等目录的检查
- ✅ 友好的输出信息（显示构建数量和跳过数量）

**命令用法**:

```bash
./xtask.sh build-js           # 增量构建（跳过最新的）
./xtask.sh build-js --force   # 强制重新构建所有
./xtask.sh clean-js           # 清理构建产物
./xtask.sh e2e                # 自动构建 JS + 运行测试
```

---

#### ~~任务 1.1.4: 清理废弃的测试文件~~ ✅ 已完成 (2026-01-26)

**文件**: `projects/cli/assets/test/` (已删除)

**实现内容**:

- ✅ 将 9 处单元测试全部改用 `tempfile` 动态生成配置
- ✅ 删除 `projects/cli/assets/test/` 目录及所有静态测试文件
- ✅ 统一测试方式：所有测试（单元测试 + E2E）都使用临时文件
- ✅ 改进的文件：
  - `bridge/manager/config_bridge.rs`
  - `bridge/manager/mod.rs`
  - `bridge/rpc/tests.rs` (4 处)
  - `bridge/runtime/tests.rs`
  - `bridge/transport/tests.rs` (2 处)

**测试验证**: 编译通过，所有测试可运行 ✓

---

#### ~~任务 1.3: 静态配置格式支持~~ ✅ 架构已正确实现

**说明**:

- ✅ Core 的 `ConfigManager` 已支持 .toml/.json/.yaml 等静态格式
- ✅ CLI 的 `ConfigLoader` 只负责提供 JS/TS 加载能力（通过 `create_js_loader()` / `create_ts_loader()`）
- ✅ 架构分工正确：Core 负责配置发现和静态格式加载，CLI 仅提供动态脚本加载闭包

**重构完成时间**: 2026-01-26

---

### 优先级 P3: 性能优化 (部分)

#### ~~任务 3.2: 配置缓存~~ ✅ Core 已实现

**说明**:

- ✅ Core 的 `ConfigManager` 已实现基于 mtime 的缓存（`get_override_config()` + `del_override_config_cache()`）
- ✅ 支持热重载：修改配置文件后调用 `del_override_config_cache()` 清除缓存
- ✅ 线程安全：使用 `Arc<RwLock<HashMap<PathBuf, OverridableConfig>>>`

**文件**: [projects/core/src/state/manager/config.rs](projects/core/src/state/manager/config.rs#L13-L15)

---

### 配置文件改进 (2026-02-02)

#### ~~任务: ESLint 和 Prettier 配置优化~~ ✅ 已完成

##### 问题 1: ESLint 忽略根目录配置文件

- **现象**: `eslint ./eslint.config.mts` 报告 "File ignored because no matching configuration was supplied"
- **原因**: `path.relative(__dirname, '.')` 返回空字符串，导致文件匹配模式为 `/**/*.{ts,cts,mts}`，无法匹配根目录文件
- **解决方案**:

```typescript
// 修改前
const commonTsFile = packages.flatMap((p) => [`${path.relative(__dirname, p.dir)}/**/*.{ts,cts,mts}`])

// 修改后
const getRootAndSrcTS = (root: string) => [`${root}/src/**/*.{ts,cts,mts}`, `${root}/*.{ts,cts,mts}`]
const commonTsFile = packages.flatMap((p) => getRootAndSrcTS(path.relative(__dirname, p.dir) || '.'))
```

- **文件**: [eslint.config.mts](../../eslint.config.mts#L42-L44)

##### 问题 2: Prettier 配置使用手动 YAML 解析

- **改进前**:
  - 手动 `fs.readFileSync` + `parseYAML` 解析 `pnpm-lock.yaml`
  - 硬编码文件路径
  - 使用 `any` 类型访问数据
- **改进后**: 使用官方 pnpm API

```typescript
import { findWorkspaceDir } from '@pnpm/find-workspace-dir'
import { getLockfileImporterId, readWantedLockfile } from '@pnpm/lockfile-file'

const workspaceRoot = await findWorkspaceDir(import.meta.dirname)
const lockfile = await readWantedLockfile(workspaceRoot, { ignoreIncompatible: false })
const importerId = getLockfileImporterId(workspaceRoot, import.meta.dirname)
const typescriptVer = lockfile.importers[importerId].devDependencies?.['typescript'] ?? '5.0.0'
```

- **优势**:
  - ✅ 类型安全（官方类型定义）
  - ✅ 格式兼容（自动处理 lockfile v5/v6/v9）
  - ✅ 自动查找 workspace 根目录
  - ✅ 官方维护，跟随 pnpm 更新
- **新增依赖**: `@pnpm/find-workspace-dir`, `@pnpm/lockfile-file`
- **文件**: [prettier.config.mts](../../prettier.config.mts#L1-L26)

##### 问题 3: 缺少 TypeScript 导入规范规则

- **添加规则**: `@typescript-eslint/consistent-type-imports`
- **配置**:

```typescript
'@typescript-eslint/consistent-type-imports': [
  'error',
  {
    prefer: 'type-imports',
    fixStyle: 'inline-type-imports',
  },
]
```

- **效果**: 强制 type-only imports 使用 `type` 关键字（如 `import type { ProjectId } from '@pnpm/lockfile-file'`）
- **文件**: [eslint.config.mts](../../eslint.config.mts#L97-L103)

##### 依赖包清单

- ✅ `@eslint/js` - 基础推荐规则
- ✅ `@eslint/eslintrc` - FlatCompat（用于转换旧配置）
- ✅ `eslint-config-prettier` - 关闭与 Prettier 冲突的规则
- ✅ `eslint-plugin-prettier` - Prettier 作为 ESLint 规则
- ✅ `typescript-eslint` - TypeScript 严格规则
- ✅ `eslint-plugin-eslint-comments` - 已安装但未使用（可选）
