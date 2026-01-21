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

pub fn build_single(target: Option<String>) -> Result<()> {
  let target = if let Some(name) = target {
    CliTarget::from_name(&name)?
  } else {
    // Detect current platform
    let platform = detect_platform()?;
    CliTarget::from_name(&platform)?
  };

  println!("🚀 Building NovelSaga CLI for {}...", target.name);

  let cli_out = cli_out_dir().join(target.name);
  let cli_dir = cli_project_dir();
  let target_dir = target_dir();

  // Create output directory
  fs::create_dir_all(&cli_out)?;

  println!("🔨 Building for {}...", target.cargo_target);

  // Build using cargo-zigbuild
  run_command(
    Command::new("cargo")
      .args([
        "zigbuild",
        "--release",
        "--target",
        target.cargo_target,
        "--target-dir",
        target_dir.to_str().unwrap(),
      ])
      .current_dir(&cli_dir),
  )?;

  // Copy binary
  let artifact = target_dir
    .join(target.cargo_target)
    .join("release")
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

pub fn build_all() -> Result<()> {
  println!("🚀 Building NovelSaga CLI for all platforms...");

  let cli_out = cli_out_dir();
  if cli_out.exists() {
    fs::remove_dir_all(&cli_out)?;
  }
  fs::create_dir_all(&cli_out)?;

  for target in CliTarget::ALL {
    println!("\n════════════════════════════════════════");
    build_single(Some(target.name.to_string()))?;
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
