# Plan: 添加 pnpm hash 自动更新 CI（四平台支持）

## TL;DR

将现有的单平台 pnpm hash CI 扩展为支持四平台（x86_64-linux、aarch64-linux、x86_64-darwin、aarch64-darwin）。

## 已完成的工作 ✅

- [x] `.github/workflows/update-pnpm-hash.yml` - 已创建（单平台版本）
- [x] `nix/packages.nix` - 已创建（从根目录移动）
- [x] `nix/hashes.json` - 已创建（单平台结构）
- [x] `flake.nix` - 已更新导入路径
- [x] 根目录 `packages.nix` - 已删除

## 剩余任务

### 任务 1: 更新 CI 支持四平台

**文件**: `.github/workflows/update-pnpm-hash.yml`

**需要做的更改**:

1. 添加矩阵策略支持 4 个平台：
   - `x86_64-linux` (ubuntu-latest)
   - `aarch64-linux` (ubuntu-24.04-arm)
   - `x86_64-darwin` (macos-13)
   - `aarch64-darwin` (macos-latest)
2. 拆分为两个 job：
   - `compute-hash`: 矩阵 job，每个平台计算自己的 hash
   - `update-hashes`: 收集所有 hash 后统一更新并提交
3. 使用 artifact 在各平台间传递 hash
4. 更新 jq 命令以支持动态 platform key

**参考**: opencode 的 `.github/workflows/nix-hashes.yml`

### 任务 2: 更新 hashes.json 为四平台结构

**文件**: `nix/hashes.json`

**当前内容**:

```json
{
  "pnpmDeps": {
    "x86_64-linux": "sha256-eXdng0f5qaS1HYerv4HYyDm4A5blZkLqUblRFic8edE="
  }
}
```

**目标内容**:

```json
{
  "pnpmDeps": {
    "x86_64-linux": "sha256-eXdng0f5qaS1HYerv4HYyDm4A5blZkLqUblRFic8edE=",
    "aarch64-linux": "sha256-PLACEHOLDER",
    "x86_64-darwin": "sha256-PLACEHOLDER",
    "aarch64-darwin": "sha256-PLACEHOLDER"
  }
}
```

### 任务 3: 更新 packages.nix 读取对应平台的 hash

**文件**: `nix/packages.nix`

**当前代码** (约第 27 行):

```nix
hash = (builtins.fromJSON (builtins.readFile ./hashes.json)).pnpmDeps."x86_64-linux";
```

**需要改为**:

```nix
hash = (builtins.fromJSON (builtins.readFile ./hashes.json)).pnpmDeps.${pkgs.system};
```

## 目标架构

```
nsaga/
├── nix/
│   ├── packages.nix      # 读取对应平台的 hash
│   └── hashes.json       # 四平台 hash 存储
├── .github/
│   └── workflows/
│       └── update-pnpm-hash.yml  # 四平台矩阵 CI
└── flake.nix             # 已更新
```

## 技术参考

**opencode 的 CI 结构**:

- 使用 `strategy.matrix` 定义多平台
- `compute-hash` job: 各平台并行计算 hash，上传 artifact
- `update-hashes` job: 下载所有 artifact，合并更新 hashes.json
- 使用 `jq --arg sys "$SYSTEM" '.nodeModules[$sys] = $hash'` 动态更新

**关键命令**:

```bash
# 动态更新 JSON
jq --arg sys "$SYSTEM" --arg h "$HASH" '.pnpmDeps[$sys] = $h' nix/hashes.json
```
