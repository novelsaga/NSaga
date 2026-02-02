//! JS Bridge 自动构建任务
//!
//! 检测并构建 CLI 所需的 JavaScript bridges

use std::{fs, path::Path, process::Command};

use anyhow::{Context, Result};

use crate::tasks::utils::project_root;

/// Bridge 配置
struct BridgeConfig {
  /// Bridge 名称
  name: &'static str,
  /// 源代码目录（相对于 workspace root）
  src_dir: &'static str,
}

const BRIDGES: &[&[BridgeConfig]] = &[
  // 第一层：基础依赖（必须先构建）
  &[BridgeConfig {
    name: "bridge-core",
    src_dir: "projects/cli-js-bridges/bridge-core",
  }],
  // 第二层：运行时适配器（可并行构建）
  &[
    BridgeConfig {
      name: "bridge-nodejs",
      src_dir: "projects/cli-js-bridges/bridge-nodejs",
    },
    BridgeConfig {
      name: "bridge-deno",
      src_dir: "projects/cli-js-bridges/bridge-deno",
    },
    BridgeConfig {
      name: "bridge-bun",
      src_dir: "projects/cli-js-bridges/bridge-bun",
    },
  ],
  // 第三层：最终产物（依赖上面所有的包）
  &[BridgeConfig {
    name: "config-bridge",
    src_dir: "projects/cli-js-bridges/config-bridge",
  }],
];

/// 构建所有 JS bridges
pub fn build_all() -> Result<()> {
  println!("🔨 Building JavaScript bridges...\n");

  let workspace_root = project_root();

  // 串行构建每一层，层内并行构建
  for (layer_idx, layer) in BRIDGES.iter().enumerate() {
    println!("📦 Building layer {} ({} bridges)...", layer_idx + 1, layer.len());

    // 使用 thread::scope 实现并行构建
    std::thread::scope(|s| {
      let handles: Vec<_> = layer
        .iter()
        .map(|bridge| {
          let workspace_root = &workspace_root;
          s.spawn(move || build_bridge(workspace_root, bridge))
        })
        .collect();

      // 等待所有线程完成并收集结果
      let results: Vec<Result<()>> = handles.into_iter().map(|h| h.join().unwrap()).collect();

      // 检查是否有失败
      for result in results {
        result?;
      }

      Ok::<(), anyhow::Error>(())
    })?;

    println!("   ✅ Layer {} complete\n", layer_idx + 1);
  }

  println!("{}", "=".repeat(60));
  println!("✅ Bridge build complete!");
  println!("{}", "=".repeat(60));

  // 复制 assets 到 out/cli/assets
  copy_assets_to_out()?;

  Ok(())
}

/// 只复制已构建的 assets 到 out/cli（不构建）
pub fn copy_only() -> Result<()> {
  println!("📦 Copying pre-built assets to out/cli/assets...\n");
  copy_assets_to_out()?;
  println!("\n✅ Assets copied successfully!");
  Ok(())
}

/// 强制重新构建所有 bridges（忽略 mtime 检查）
pub fn build_all_force() -> Result<()> {
  // 现在和 build_all 相同，保留作为别名
  build_all()
}

/// 构建单个 bridge
fn build_bridge(workspace_root: &Path, config: &BridgeConfig) -> Result<()> {
  let src_dir = workspace_root.join(config.src_dir);

  // 检查源目录是否存在
  if !src_dir.exists() {
    println!("⚠️  {} source directory not found, skipping", config.name);
    return Ok(());
  }

  println!("🔨 Building {}...", config.name);

  // 运行 pnpm run build
  let status = Command::new("pnpm")
    .args(["run", "build"])
    .current_dir(&src_dir)
    .status()
    .with_context(|| format!("Failed to run pnpm build for {}", config.name))?;

  if !status.success() {
    anyhow::bail!("Failed to build {}", config.name);
  }

  println!("   ✅ {} built successfully", config.name);
  Ok(())
}

/// 复制 assets 到 out/cli/assets
fn copy_assets_to_out() -> Result<()> {
  use crate::tasks::utils::cli_out_dir;

  println!("\n📦 Copying assets to out/cli/assets...");

  let workspace_root = project_root();
  let src_dir = workspace_root.join("projects/cli/assets/js/dist");
  let dest_dir = cli_out_dir().join("assets/js/dist");

  // 创建目标目录
  fs::create_dir_all(&dest_dir).with_context(|| format!("Failed to create directory: {}", dest_dir.display()))?;

  // 复制所有文件（不只是 .js）
  if src_dir.exists() {
    for entry in fs::read_dir(&src_dir)? {
      let entry = entry?;
      let path = entry.path();
      if path.is_file() {
        let file_name = path.file_name().unwrap();
        let dest_file = dest_dir.join(file_name);
        fs::copy(&path, &dest_file)
          .with_context(|| format!("Failed to copy {} to {}", path.display(), dest_file.display()))?;
        println!("   ✅ Copied {}", file_name.to_string_lossy());
      }
    }
  } else {
    println!("   ⚠️  Source directory not found: {}", src_dir.display());
  }

  println!("   📁 Assets copied to: {}", dest_dir.display());

  Ok(())
}

/// 清理所有构建产物
pub fn clean() -> Result<()> {
  println!("🧹 Cleaning JavaScript bridge artifacts...\n");

  let workspace_root = project_root();
  let dist_dir = workspace_root.join("projects/cli/assets/js/dist");

  if dist_dir.exists() {
    fs::remove_dir_all(&dist_dir).context("Failed to remove dist directory")?;
    println!("   ✅ Removed {}", dist_dir.display());
  } else {
    println!("   ℹ️  Dist directory does not exist, nothing to clean");
  }

  println!("\n✅ Bridge artifacts cleaned!");

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bridge_config() {
    // 确保至少有一个层级
    assert!(!BRIDGES.is_empty());

    // 检查每一层的配置
    for layer in BRIDGES {
      assert!(!layer.is_empty(), "Each layer should have at least one bridge");
      for bridge in *layer {
        assert!(!bridge.name.is_empty());
        assert!(!bridge.src_dir.is_empty());
      }
    }
  }
}
