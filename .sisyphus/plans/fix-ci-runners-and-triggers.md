# Plan: 修复 CI Android NDK 错误和优化 bundle 平台支持

## TL;DR

基于 Playwright 查看的 GitHub Actions 日志分析：

- Android NDK 错误是因为 `bundle` target 包含 `pkgsCross.aarch64-android-prebuilt.stdenv.cc`
- Android NDK 仅支持 x86_64-linux 构建，在其他平台（ARM Linux、macOS）会报错
- 需要创建专门的 pnpm hash updater target 避免评估 Android 依赖
- bundle package 应该只在 x86_64-linux 上可用

## 问题分析（已完成调查）

### Android NDK 错误 ✅ 已确认

**错误信息**：

- aarch64-linux: `Android NDK doesn't support building on aarch64-unknown-linux-gnu`
- x86_64-darwin: `Android NDK doesn't support building on arm64-apple-darwin`

**根本原因**：

- workflow 使用 `nix build ".#bundle"` 触发 hash 不匹配
- bundle 包含 `pkgsCross.aarch64-android-prebuilt.stdenv.cc`
- Nix 评估时会尝试评估所有 nativeBuildInputs
- Android NDK 仅支持 x86_64-linux 构建

## 待执行的任务

### 任务 1: 创建 pnpmHashUpdater target

**文件**: `nix/packages.nix`

**添加内容**（在 `commonArgs` 之后）：

```nix
  # 专门用于计算 pnpm hash 的 target，避免评估 Android 依赖
  pnpmHashUpdater = pkgs.stdenvNoCC.mkDerivation {
    pname = "pnpm-hash-updater";
    inherit (commonArgs) version src;

    nativeBuildInputs = [
      pkgs.nodejs
      pkgs.pnpm
    ];

    buildPhase = ''
      export HOME=$TMPDIR
      export PNPM_HOME=$TMPDIR/pnpm
      mkdir -p $PNPM_HOME

      # 使用 pnpm fetch 获取依赖
      pnpm fetch --frozen-lockfile
    '';

    installPhase = ''
      touch $out
    '';

    # 关键：不依赖任何平台特定的工具链
    dontFixup = true;
  };
```

### 任务 2: bundle package 限制为 x86_64-linux

**文件**: `nix/packages.nix`

**修改 `bundle` 定义**（约第 92 行）：

将：

```nix
    bundle = pkgs.rustPlatform.buildRustPackage (
      commonArgs
      // {
        pname = "novelsaga-bundle";
```

改为：

```nix
    bundle = if pkgs.system == "x86_64-linux" then pkgs.rustPlatform.buildRustPackage (
      commonArgs
      // {
        pname = "novelsaga-bundle";
```

并在 bundle 定义结尾添加：

```nix
      }
    ) else null;
```

### 任务 3: 更新 workflow 使用 pnpmHashUpdater

**文件**: `.github/workflows/update-pnpm-hash.yml`

**修改 "Compute pnpm hash" 步骤**：

将：

```bash
          # Build bundle to extract real hash
          nix build ".#bundle" --no-link 2>&1 | tee "$BUILD_LOG" || true
```

改为：

```bash
          # Build pnpmHashUpdater to extract real hash
          nix build ".#pnpmHashUpdater" --no-link 2>&1 | tee "$BUILD_LOG" || true
```

### 任务 4: 更新 flake.nix 导出 pnpmHashUpdater

**文件**: `flake.nix`

**在 packages 导出中添加**：

```nix
packages = {
  default = ...;
  cli = ...;
  bundle = ...;
  pnpmHashUpdater = ...;  # 添加这一行
};
```

## 验证清单

- [ ] `nix build .#pnpmHashUpdater` 在 x86_64-linux 上成功
- [ ] `nix build .#pnpmHashUpdater` 在 aarch64-linux 上成功（无 Android 错误）
- [ ] `nix build .#pnpmHashUpdater` 在 x86_64-darwin 上成功（无 Android 错误）
- [ ] `nix build .#pnpmHashUpdater` 在 aarch64-darwin 上成功（无 Android 错误）
- [ ] `nix build .#bundle` 在 x86_64-linux 上仍然可用
- [ ] `nix build .#bundle` 在其他平台返回 null 或报错信息友好
