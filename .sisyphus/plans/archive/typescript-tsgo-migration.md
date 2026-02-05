# TypeScript 混合迁移计划（tsgo + tsc）

## TL;DR

将项目配置为**混合使用 tsgo 和 tsc**：

- **tsgo**：用于 Prettier/Eslint 等代码检查工具（快速类型检查，8-10 倍性能提升）
- **tsc**：保留用于构建和声明文件生成（稳定可靠）
- **统一使用 pnpm exec**：替换所有 `npx` 调用

**关键变更**：

1. 安装 `@typescript/native-preview` 和 `semver`
2. 修改 `prettier.config.mts` 使用 semver 解析版本号（去掉 tsgo 后缀）
3. 将 `build-tool/src/index.mts` 中的 `npx tsc` 改为 `pnpm exec tsc`

---

## Context

### 原始需求

用户希望从 tsc 迁移到 tsgo（@typescript/native-preview），但：

- @ianvs/prettier-plugin-sort-imports 依赖 TypeScript 版本配置
- tsgo 版本号为 `7.0.0-dev.20260205.1` 格式，需要去掉后缀
- 需要统一使用 `pnpm exec` 替代 `npx`

### 调研结论

根据 tsgo 官方文档和 GitHub issues：

- tsgo 的 **Declaration Emit 状态为 `in progress`**
- 存在已知问题（如 `emitDeclarationOnly` 时可能挂起）
- **建议**：tsgo 用于类型检查，tsc 保留用于声明生成

---

## Work Objectives

### Core Objective

配置项目混合使用 tsgo 和 tsc，优化开发体验的同时保持构建稳定性。

### Concrete Deliverables

- [ ] 安装 `@typescript/native-preview` 和 `semver`
- [ ] 修改 `prettier.config.mts` 使用 semver 解析版本号
- [ ] 将 `build-tool/src/index.mts` 中的 `npx tsc` 改为 `pnpm exec tsc`

### Definition of Done

- [ ] `pnpm install` 成功安装新依赖
- [ ] `pnpm exec prettier --check .` 正常工作
- [ ] `./xtask.sh build-js` 构建成功
- [ ] 项目中无 `npx` 或 `pnpx` 调用

### Must Have

- 保留 `typescript` 包用于构建
- 使用 `semver` 正确解析版本号
- 统一使用 `pnpm exec`

### Must NOT Have

- 不修改 eslint 配置（保持现状）
- 不修改构建流程（继续使用 tsc 生成声明）
- 不删除任何现有依赖

---

## Verification Strategy

### Test Decision

- **Infrastructure exists**: YES
- **Automated tests**: Tests-after
- **Framework**: cargo test

### Agent-Executed QA Scenarios

**Scenario 1: Prettier 配置验证**
Tool: Bash
Preconditions: 完成所有修改并运行 `pnpm install`
Steps: 1. 运行 `pnpm exec prettier --check .` 2. 验证无配置错误
Expected Result: Prettier 检查通过或正常报告格式问题
Evidence: 终端输出

**Scenario 2: 构建验证**
Tool: Bash
Preconditions: 完成所有修改
Steps: 1. 运行 `./xtask.sh build-js` 2. 验证所有 bridge 包构建成功 3. 验证 `dist/` 目录生成正确的 `.d.ts` 文件
Expected Result: 构建成功，声明文件正确生成
Evidence: 构建输出日志

**Scenario 3: tsgo 安装验证**
Tool: Bash
Preconditions: 完成 `pnpm install`
Steps: 1. 运行 `pnpm exec tsgo --version` 2. 验证输出版本号
Expected Result: 显示 tsgo 版本（如 `7.0.0-dev.20260205.1`）
Evidence: 终端输出版本号

**Scenario 4: 检查无 npx 调用**
Tool: Bash
Preconditions: 完成所有修改
Steps: 1. 运行 `grep -r "npx" --include="*.mts" --include="*.ts" --include="*.json" projects/` 2. 验证无 `npx` 调用（除了 node_modules）
Expected Result: 无匹配结果（或只有 node_modules 中的）
Evidence: grep 输出为空

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Start Immediately):
├── Task 1: 修改 package.json（添加依赖）
└── Task 2: 修改 prettier.config.mts（版本解析）

Wave 2 (After Wave 1):
└── Task 3: 修改 build-tool/src/index.mts（npx → pnpm exec）

Wave 3 (After Wave 2):
└── Task 4: 安装依赖并验证
```

### Dependency Matrix

| Task | Depends On | Blocks | Can Parallelize With |
| ---- | ---------- | ------ | -------------------- |
| 1    | None       | 4      | 2                    |
| 2    | None       | 4      | 1                    |
| 3    | None       | 4      | 1, 2                 |
| 4    | 1, 2, 3    | None   | None                 |

---

## TODOs

- [ ] 1. 修改 package.json 添加依赖

  **What to do**:
  - 添加 `"@typescript/native-preview": "^7.0.0-dev"`
  - 添加 `"semver": "^7.6.3"`
  - 保留 `"typescript": "^5.9.3"`

  **Must NOT do**:
  - 不要删除 `typescript` 依赖
  - 不要修改其他依赖版本

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `git-master`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 4
  - **Blocked By**: None

  **References**:
  - `package.json` - 当前依赖列表

  **Acceptance Criteria**:
  - [ ] `@typescript/native-preview` 添加到 devDependencies
  - [ ] `semver` 添加到 devDependencies
  - [ ] `typescript` 仍然保留在 devDependencies

  **Commit**: YES
  - Message: `chore(deps): add @typescript/native-preview and semver`
  - Files: `package.json`

- [ ] 2. 修改 prettier.config.mts 使用 semver 解析版本号

  **What to do**:
  - 导入 `semver` 的 `parse` 函数
  - 修改版本号解析逻辑，去掉后缀（如 `-dev.20260205.1`）

  **Must NOT do**:
  - 不要修改其他配置
  - 不要删除现有导入

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `ts-bridge`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 4
  - **Blocked By**: None

  **References**:
  - `prettier.config.mts` - 当前配置
  - semver 文档 - parse 函数用法

  **Acceptance Criteria**:
  - [ ] 导入 `parse` from `semver`
  - [ ] 使用 semver 解析版本号
  - [ ] 生成的版本号为 `major.minor.patch` 格式

  **Code Change**:

  ```typescript
  // 添加导入
  import { parse } from 'semver'

  // 修改版本解析逻辑（第 26 行附近）
  const rawVersion = rootImporter.devDependencies?.['typescript'] ?? '5.0.0'
  const parsed = parse(rawVersion)
  const typescriptVer = parsed ? `${parsed.major}.${parsed.minor}.${parsed.patch}` : '5.0.0'
  ```

  **Commit**: YES
  - Message: `refactor(config): use semver to parse TypeScript version`
  - Files: `prettier.config.mts`

- [ ] 3. 修改 build-tool/src/index.mts 替换 npx 为 pnpm exec

  **What to do**:
  - 将 `npx tsc --emitDeclarationOnly` 改为 `pnpm exec tsc --emitDeclarationOnly`

  **Must NOT do**:
  - 不要修改其他逻辑
  - 不要改变 tsc 参数

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `ts-bridge`

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1
  - **Blocks**: Task 4
  - **Blocked By**: None

  **References**:
  - `build-tool/src/index.mts` - 第 115 行

  **Acceptance Criteria**:
  - [ ] `npx tsc` 改为 `pnpm exec tsc`

  **Code Change**:

  ```typescript
  // 第 115 行
  await execAsync('pnpm exec tsc --emitDeclarationOnly', { cwd: rootDir })
  ```

  **Commit**: YES
  - Message: `refactor(build): use pnpm exec instead of npx`
  - Files: `projects/cli-js-bridges/build-tool/src/index.mts`

- [ ] 4. 安装依赖并验证

  **What to do**:
  - 运行 `pnpm install` 安装新依赖
  - 验证 prettier 配置正确
  - 验证构建流程正常
  - 验证 tsgo 可用

  **Must NOT do**:
  - 不要跳过验证步骤

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: `nix-workflow`

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: None
  - **Blocked By**: Task 1, 2, 3

  **Acceptance Criteria**:
  - [ ] `pnpm install` 成功完成
  - [ ] `pnpm exec prettier --check .` 正常执行
  - [ ] `pnpm exec tsgo --version` 显示版本号
  - [ ] `./xtask.sh build-js` 构建成功
  - [ ] `grep -r "npx" --include="*.mts" --include="*.ts" projects/` 无结果

  **Commit**: NO（验证步骤，不提交）

---

## Commit Strategy

| After Task | Message                                                    | Files                      |
| ---------- | ---------------------------------------------------------- | -------------------------- |
| 1          | `chore(deps): add @typescript/native-preview and semver`   | `package.json`             |
| 2          | `refactor(config): use semver to parse TypeScript version` | `prettier.config.mts`      |
| 3          | `refactor(build): use pnpm exec instead of npx`            | `build-tool/src/index.mts` |

---

## Success Criteria

### Verification Commands

```bash
# 1. 验证 prettier
pnpm exec prettier --check .

# 2. 验证 tsgo 安装
pnpm exec tsgo --version

# 3. 验证构建
./xtask.sh build-js

# 4. 验证无 npx 调用
grep -r "npx" --include="*.mts" --include="*.ts" projects/cli-js-bridges/ || echo "No npx found"
```

### Final Checklist

- [ ] 所有 "Must Have" 已完成
- [ ] 所有 "Must NOT Have" 已避免
- [ ] 所有验证命令通过
- [ ] 项目中无 `npx` 或 `pnpx` 调用

---

## Notes

### 为什么保留 tsc？

根据 tsgo 官方文档，Declaration Emit 状态为 `in progress`，存在已知问题：

- Issue #972: 错误时不生成声明文件（已修复）
- Issue #2265: sourcemap 性能问题
- Issue #2678: `emitDeclarationOnly` 时可能挂起

因此建议：

- **tsgo**：用于快速类型检查（`--noEmit`）
- **tsc**：保留用于构建和声明生成

### 为什么统一使用 pnpm exec？

- `pnpm exec` 是 pnpm 的标准命令，与项目包管理器一致
- `npx` 可能使用不同的包解析策略
- `pnpx` 已被 pnpm 弃用
