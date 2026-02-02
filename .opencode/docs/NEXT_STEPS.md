# NovelSaga CLI - 下一步工作计划

**更新时间**: 2026-02-02
**当前状态**: ✅ P0 构建验证完成 + Nix 构建支持完成 + 构建系统重构完成

---

## 当前进度总览

### ✅ 已完成的核心功能

#### Stage 4: Bridge Manager (2026-01-26 完成)

- ✅ JSON-RPC 2.0 通信层（Transport + RPC Client）
- ✅ 运行时管理（Node.js/Bun/Deno 发现和进程管理）
- ✅ Bridge 管理器（懒加载、重试、线程安全）
- ✅ ConfigBridge 实现（通过 Bridge 加载 JS/TS 配置）
- ✅ 内存安全重构（`Arc<Mutex<Box<dyn Bridge>>>` 架构）

#### Stage 5: Config Loader (2026-01-26 完成)

- ✅ ConfigLoader 实现（配置查找和加载）
- ✅ CLI 集成（js_loader/ts_loader 闭包）
- ✅ E2E 测试框架（xtask test module）
- ✅ 多格式支持（.mjs/.ts/.cjs）

#### P1.1: 代码质量改进 (2026-01-26 完成)

- ✅ **常量复用**：消除硬编码，统一使用 core 的常量定义
  - 文件扩展名：`CONFIG_FILE_NAMES`, `NovelSagaFileFormat::get_extensions()`
  - 环境变量：`env_keys::*` 模块（Rust + JS 双端同步）
- ✅ **E2E 测试扩展**：从 5 个测试扩展到 14 个
  - 3 个运行时 × 4 种配置格式 = 12 个矩阵测试
  - 2 个基础测试（配置优先级、错误处理）
  - 智能跳过机制（Node.js < 23.6 跳过 TS 测试）
  - TypeScript 测试使用真实的 TS 语法（interface、类型注解）

**测试状态**:

- ✅ 单元测试：Bridge Manager、ConfigBridge
- ✅ 集成测试：`test_bridge_manager`
- ✅ E2E 测试：`./xtask.sh e2e` (14 个场景全部通过)

---

## 今天完成的工作总结 (2026-02-02)

### ✅ P0 - 构建验证与 Nix 构建支持 (已完成)

**Nix 构建完整支持**:

#### 1. **xtask 新增 `--skip-js` 和 `--copy-only` 参数**

- `./xtask.sh cli --skip-js` - 跳过 JS 构建，只构建 CLI
- `./xtask.sh cli-all --skip-js` - 跳过 JS 构建，只构建所有平台 CLI
- `./xtask.sh build-all --skip-js` - 跳过 JS 构建，构建所有产物
- `./xtask.sh build-js --copy-only` - 只复制已构建的 assets 到 `out/cli/`

**用途**: Nix 构建中手动构建 JS 后，可使用 `--skip-js` 避免重复构建

#### 2. **Nix Bundle 包 (`nix build .#bundle`)**

**依赖管理**:

- 使用 `fetchPnpmDeps` 预取 npm 依赖（需要 hash）
- 添加 `pkgs.pnpm`, `pkgs.nodejs`, `pkgs.deno` 到 `nativeBuildInputs`
- `pnpmConfigHook` 自动安装依赖到 `node_modules`

**构建流程**:

1. `pnpmConfigHook` 安装 JS 依赖（离线，使用预取的依赖）
2. 手动运行 `node build.mts` 构建每个 bridge（避免 pnpm 自我更新触发网络请求）
   - `bridge-deno/build.mts` 自动生成 `deno.d.ts`（调用 `deno types`）
3. `cargo run -p xtask --release -- build-all --skip-js` 构建所有 Rust 目标
4. `cargo run -p xtask --release -- build-js --copy-only` 复制 assets 到 CLI 输出
5. 安装到 `$out/share/novelsaga/{cli,wasm,so}/`

**产物结构**:

```
result/share/novelsaga/
├── cli/
│   ├── linux-x64/novelsaga
│   ├── macos-arm64/novelsaga
│   ├── windows-x64/novelsaga.exe
│   └── assets/js/dist/
│       ├── config-bridge.js
│       └── package.json
├── wasm/
│   └── novelsaga_core_bg.wasm
└── so/
    ├── linux-x64/libnovelsaga_core.so
    └── ...
```

#### 3. **Nix Default 包 (`nix build` 或 `nix build .#cli`)**

**特点**: 只构建 CLI，产物更轻量

**构建流程**:

1. 使用 nightly Rust toolchain（支持 `#![feature(mpmc_channel)]`）
2. `pnpmConfigHook` 安装 JS 依赖
3. `preBuild`: 手动构建所有 JS bridges
4. `cargo build` 构建 CLI（`buildAndTestSubdir = "projects/cli"`）
5. `postInstall`: 复制 JS assets 到 `$out/share/novelsaga/assets/js/dist/`

**产物结构**:

```
result/
├── bin/
│   └── novelsaga
└── share/
    └── novelsaga/
        └── assets/
            └── js/
                └── dist/
                    ├── config-bridge.js
                    └── package.json
```

**验证结果**:

- ✅ `nix build .#bundle` 成功构建所有平台产物
- ✅ `nix build` 或 `nix build .#cli` 成功构建单平台 CLI
- ✅ JS assets 正确复制到输出目录
- ✅ 所有构建在 Nix 沙盒中完成（无网络访问）

**pnpmDeps hash 更新流程**:

```bash
# 1. 修改 hash 为占位符
hash = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

# 2. 运行构建获取正确的 hash
nix build .#bundle 2>&1 | grep "got:"

# 3. 更新 hash 到 packages.nix
```

### ✅ P1.2 - 构建系统重构 (2026-02-02 完成)

**问题**: 5 个 bridge 包的 build.mts 存在大量重复代码（50-68 行相似逻辑）

**解决方案**: 创建统一的构建工具包 `@nsaga/build-tool`

**实施内容**:

1. **创建 build-tool 包** (`projects/cli-js-bridges/build-tool/`):
   - `buildBridge()` 函数提供统一构建接口
   - 支持选项：
     - `name`: 包名
     - `rootDir`: 根目录
     - `entryPoints`: 入口文件列表（或使用 `autoScanSrc`）
     - `autoScanSrc`: 自动扫描 src 目录所有 .ts 文件
     - `preBuild`: 构建前钩子（如 Deno types 生成）
     - `postBuild`: 构建后钩子（如创建 package.json）
     - `esbuildOverrides`: 自定义 esbuild 配置
     - `generateDts`: 是否生成 .d.ts 文件（默认 true）

2. **重构所有 bridge 构建脚本**:
   - **bridge-core** (68 → 7 行): 使用 `autoScanSrc: true`
   - **bridge-nodejs** (58 → 7 行): 使用 `autoScanSrc: true`
   - **bridge-bun** (50 → 11 行): 指定 `entryPoints`
   - **bridge-deno** (58 → 23 行): 指定 `entryPoints` + `preBuild` hook
   - **config-bridge** (49 → 27 行): `generateDts: false` + 自定义输出路径

3. **Workspace 配置**:
   - 添加 build-tool 到所有 bridge 的 `devDependencies`
   - 使用包名引用 `@nsaga/build-tool` 而非相对路径
   - pnpm workspace 自动包含（`projects/cli-js-bridges/*`）

**重构效果**:

- ✅ 消除了约 **180 行重复代码**
- ✅ 统一的构建逻辑，易于维护
- ✅ 支持灵活的扩展（hooks + overrides）
- ✅ 所有构建测试通过：`./xtask.sh build-js` ✅

**代码示例**:

```typescript
// 旧代码：50+ 行 esbuild 配置和错误处理
// 新代码：
import { buildBridge } from '@nsaga/build-tool'
buildBridge({
  name: 'bridge-bun',
  rootDir: __dirname,
  entryPoints: ['src/transport.ts', 'src/index.ts'],
})
```

---

## 接下来的工作计划 (2026-02-03 起)

### 第 1 步：代码清理与规范化 (P0 必须完成)

**任务清单**:

1. **Rust 代码清理 (Clippy)**:
   - [ ] 运行 `cargo clippy --all-targets --all-features` 查看所有警告
   - [ ] 逐个修复 `unused imports`, `dead code`, `deprecated` 等警告
   - [ ] 确保所有 crate 通过 clippy 检查

2. **冗余代码删除**:
   - [ ] 删除临时测试代码、注释掉的废弃逻辑
   - [ ] 删除未使用的函数、结构体、模块

3. **TypeScript/JavaScript 代码规范化**:
   - [ ] 为 `projects/cli-js-bridges` 配置 Prettier
   - [ ] 配置 ESLint 规则
   - [ ] 运行格式化并修复 lint 错误

**说明**: 这些属于代码清理（Code Cleanup）和规范化（Linting），而非重构（Refactoring）

### 第 2 步：错误消息优化 (P1 优化体验)

改进用户友好的错误提示，例如：

- ❌ 当前: `IO Error`
- ✅ 改进: `未找到 Node.js，请检查 PATH 或使用 --node-path 参数`

**重点文件**:

- `projects/cli/src/bridge/error.rs`
- `projects/cli/src/config/error.rs`
- `projects/cli/src/assets/error.rs`

### 第 3 步：下一阶段架构分析 (P2 核心功能)

**决策点**: 元数据管理器 (Metadata) vs 插件加载器 (Plugin Loader)

**分析**:

- 插件系统可能依赖元数据结构
- 需要详细的需求分析，决定并行或顺序开发

---

## 已完成的里程碑

### ✅ 构建验证 (2026-02-02)

- [x] **Manual Build**: `./xtask.sh build-all` ✅
- [x] **Nix Build**: `nix build .#bundle` 和 `nix build .#cli` ✅

### ✅ 构建系统重构 (2026-02-02)

- [x] **Build Tool 统一**: 创建 `@nsaga/build-tool` 包 ✅
- [x] **消除重复代码**: 5 个 bridge 构建脚本重构完成 ✅
- [x] **构建测试**: `./xtask.sh build-js` 验证通过 ✅

### ⏳ 待完成：代码清理与规范化

- [ ] **Rust Clippy**: 清理所有 clippy 警告
- [ ] **删除冗余代码**: 临时代码和未使用的定义
- [ ] **JS/TS 规范化**: Prettier + ESLint 配置和应用

---

## 暂缓任务 (已移至 BACKLOG.md)

以下非紧急任务已移出当前迭代:

- 稳健性: RPC 超时、Bridge 崩溃日志收集
- 体验: 配置热重载
- 性能: Bridge 预热 (Warmup)
- 运维: CI/CD 流水线

---

## 历史完成记录

完整历史记录请参见：[COMPLETED_TASKS.md](COMPLETED_TASKS.md)

- P1.3 静态配置格式支持 (Done)
- P1.1.4 清理废弃测试 (Done)
- P1.1.2 JS Bridge 自动构建 (Done)
- P1.1.1 用户指定运行时 (Done)
