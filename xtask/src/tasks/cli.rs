use std::{fs, process::Command};

use anyhow::Result;

use super::utils::*;

#[derive(Debug)]
pub struct CliTarget {
  pub name: &'static str,
  pub cargo_target: &'static str,
  pub binary_name: &'static str,
}

impl CliTarget {
  pub const ALL: &'static [Self] = &[
    Self::MACOS_ARM64,
    Self::MACOS_X64,
    Self::WINDOWS_ARM64,
    Self::WINDOWS_X64,
    Self::LINUX_ARM64,
    Self::LINUX_X64,
  ];
  pub const LINUX_ARM64: Self = Self {
    name: "linux-arm64",
    cargo_target: "aarch64-unknown-linux-gnu",
    binary_name: "novelsaga",
  };
  pub const LINUX_X64: Self = Self {
    name: "linux-x64",
    cargo_target: "x86_64-unknown-linux-gnu",
    binary_name: "novelsaga",
  };
  pub const MACOS_ARM64: Self = Self {
    name: "macos-arm64",
    cargo_target: "aarch64-apple-darwin",
    binary_name: "novelsaga",
  };
  pub const MACOS_X64: Self = Self {
    name: "macos-x64",
    cargo_target: "x86_64-apple-darwin",
    binary_name: "novelsaga",
  };
  pub const WINDOWS_ARM64: Self = Self {
    name: "windows-arm64",
    cargo_target: "aarch64-pc-windows-gnullvm",
    binary_name: "novelsaga.exe",
  };
  pub const WINDOWS_X64: Self = Self {
    name: "windows-x64",
    cargo_target: "x86_64-pc-windows-gnu",
    binary_name: "novelsaga.exe",
  };

  pub fn from_name(name: &str) -> Result<&'static Self> {
    Self::ALL
      .iter()
      .find(|t| t.name == name)
      .ok_or_else(|| anyhow::anyhow!("Unknown target: {}", name))
  }
}

pub fn build_single(target: Option<String>, asset_install_dir: &str, release: bool, skip_js: bool) -> Result<()> {
  // 确保 JS bridges 已构建（除非明确跳过）
  if !skip_js {
    println!("🔧 Ensuring JavaScript bridges are built...");
    super::build::build_all()?;
    println!();
  }

  build_single_internal(target, asset_install_dir, release)
}

/// 内部构建函数（不检查 JS bridges）
fn build_single_internal(target: Option<String>, asset_install_dir: &str, release: bool) -> Result<()> {
  let target = if let Some(name) = target {
    CliTarget::from_name(&name)?
  } else {
    // Detect current platform
    let platform = detect_platform()?;
    CliTarget::from_name(&platform)?
  };

  println!("🚀 Building NovelSaga CLI for {}...", target.name);
  println!("📦 Asset install dir: {}", asset_install_dir);

  let cli_out = cli_out_dir().join(target.name);
  let cli_dir = cli_project_dir();
  let target_dir = target_dir();

  // Create output directory
  fs::create_dir_all(&cli_out)?;

  println!("🔨 Building for {}...", target.cargo_target);

  // Build command
  let mut cmd = Command::new("cargo");
  cmd.arg("zigbuild");

  if release {
    cmd.arg("--release");
  }

  cmd
    .args(["--target", target.cargo_target])
    .args(["--target-dir", target_dir.to_str().unwrap()])
    .env("NSAGA_ASSET_INSTALL_DIR", asset_install_dir)
    .current_dir(&cli_dir);

  run_command(&mut cmd)?;

  // Copy binary
  let build_profile = if release { "release" } else { "debug" };
  let artifact = target_dir
    .join(target.cargo_target)
    .join(build_profile)
    .join(target.binary_name);

  if !artifact.exists() {
    anyhow::bail!("Built binary missing: {}", artifact.display());
  }

  fs::copy(&artifact, cli_out.join(target.binary_name))?;
  println!("✅ {} -> {}", target.cargo_target, cli_out.display());

  println!("\n✅ CLI build complete!");

  if let Ok(metadata) = fs::metadata(&artifact) {
    let size = metadata.len() as f64 / 1024.0 / 1024.0;
    println!("📊 Binary size: {:.2} MB", size);
  }

  println!("\n🚀 Binary ready: {}", cli_out.join(target.binary_name).display());

  Ok(())
}

pub fn build_all(asset_install_dir: &str, release: bool, skip_js: bool) -> Result<()> {
  println!("🚀 Building NovelSaga CLI for all platforms...");
  println!("📦 Asset install dir: {}", asset_install_dir);

  // 1. 先清理旧的输出目录
  let cli_out = cli_out_dir();
  if cli_out.exists() {
    fs::remove_dir_all(&cli_out)?;
  }
  fs::create_dir_all(&cli_out)?;

  // 2. 构建 JS bridges（只构建一次，会自动复制到 out/cli/assets，除非明确跳过）
  if !skip_js {
    println!("\n🔧 Building JavaScript bridges...");
    super::build::build_all()?;
    println!();
  }

  for target in CliTarget::ALL {
    println!("\n════════════════════════════════════════");
    build_single_internal(Some(target.name.to_string()), asset_install_dir, release)?;
  }

  println!("\n════════════════════════════════════════");
  println!("✅ All platforms built successfully!");

  // Show binary sizes
  println!("\n📊 Binary sizes:");
  for target in CliTarget::ALL {
    let bin_path = cli_out.join(target.name).join(target.binary_name);
    if bin_path.exists() {
      if let Ok(metadata) = fs::metadata(&bin_path) {
        let size = metadata.len() as f64 / 1024.0 / 1024.0;
        println!("  {}: {:.2} MB", target.name, size);
      }
    }
  }

  println!("\n🚀 Binaries ready for distribution!");

  Ok(())
}
