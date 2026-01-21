use std::{fs, process::Command};

use anyhow::Result;

use super::utils::*;

#[derive(Debug)]
pub struct SoTarget {
  pub name: &'static str,
  pub cargo_target: &'static str,
  pub lib_name: &'static str,
}

impl SoTarget {
  pub const ALL: &'static [Self] = &[
    Self::MACOS_ARM64,
    Self::MACOS_X64,
    Self::WINDOWS_ARM64,
    Self::WINDOWS_X64,
    Self::LINUX_ARM64,
    Self::LINUX_X64,
    Self::ANDROID_ARM64,
  ];
  pub const ANDROID_ARM64: Self = Self {
    name: "android-arm64",
    cargo_target: "aarch64-linux-android",
    lib_name: "libnovelsaga_core.so",
  };
  pub const LINUX_ARM64: Self = Self {
    name: "linux-arm64",
    cargo_target: "aarch64-unknown-linux-gnu",
    lib_name: "libnovelsaga_core.so",
  };
  pub const LINUX_X64: Self = Self {
    name: "linux-x64",
    cargo_target: "x86_64-unknown-linux-gnu",
    lib_name: "libnovelsaga_core.so",
  };
  pub const MACOS_ARM64: Self = Self {
    name: "macos-arm64",
    cargo_target: "aarch64-apple-darwin",
    lib_name: "libnovelsaga_core.dylib",
  };
  pub const MACOS_X64: Self = Self {
    name: "macos-x64",
    cargo_target: "x86_64-apple-darwin",
    lib_name: "libnovelsaga_core.dylib",
  };
  pub const WINDOWS_ARM64: Self = Self {
    name: "windows-arm64",
    cargo_target: "aarch64-pc-windows-gnullvm",
    lib_name: "novelsaga_core.dll",
  };
  pub const WINDOWS_X64: Self = Self {
    name: "windows-x64",
    cargo_target: "x86_64-pc-windows-gnu",
    lib_name: "novelsaga_core.dll",
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
    SoTarget::from_name(&name)?
  } else {
    // Detect current platform
    let platform = detect_platform()?;
    SoTarget::from_name(&platform)?
  };

  println!("🚀 Building NovelSaga Shared Library for {}...", target.name);

  let so_out = so_out_dir().join(target.name);
  let core_dir = core_project_dir();
  let target_dir = target_dir();

  // Create output directory
  if so_out.exists() {
    fs::remove_dir_all(&so_out)?;
  }
  fs::create_dir_all(&so_out)?;
  fs::create_dir_all(so_out.join("include"))?;

  println!("🔨 Building for {}...", target.cargo_target);

  // Build dynamic library
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
      .current_dir(&core_dir),
  )?;

  // Generate C++ headers with diplomat-tool
  println!("📄 Generating C++ headers with diplomat-tool...");
  let cpp_out = so_out.join("include");

  // 使用库 API 生成 C++ 绑定
  let entry_path = core_dir.join("src/lib.rs");
  let config = diplomat_tool::config::Config::default();

  diplomat_tool::r#gen(
    &entry_path,
    "cpp",
    &cpp_out,
    &diplomat_tool::DocsUrlGenerator::default(),
    config,
    false, // not silent
  )?;

  println!("✅ C++ headers generated: {}", cpp_out.display());

  // Copy library file
  let lib_file = target_dir
    .join(target.cargo_target)
    .join("release")
    .join(target.lib_name);

  if lib_file.exists() {
    fs::copy(&lib_file, so_out.join(target.lib_name))?;
    println!("✅ Build complete!");

    if let Ok(metadata) = fs::metadata(&lib_file) {
      let size = metadata.len() as f64 / 1024.0 / 1024.0;
      println!("📊 Library size: {:.2} MB", size);
    }

    println!("\n🚀 Library ready at: {}", so_out.display());
  } else {
    anyhow::bail!("Failed to find library file at: {}", lib_file.display());
  }

  Ok(())
}

pub fn build_all() -> Result<()> {
  println!("🚀 Building NovelSaga Shared Library for all platforms...");

  let so_out = so_out_dir();
  if so_out.exists() {
    fs::remove_dir_all(&so_out)?;
  }
  fs::create_dir_all(&so_out)?;

  for target in SoTarget::ALL {
    println!("\n════════════════════════════════════════");
    build_single(Some(target.name.to_string()))?;
  }

  println!("\n════════════════════════════════════════");
  println!("✅ All platforms built successfully!");

  // Show library sizes
  println!("\n📊 Library sizes:");
  for target in SoTarget::ALL {
    let lib_path = so_out.join(target.name).join(target.lib_name);
    if lib_path.exists() {
      if let Ok(metadata) = fs::metadata(&lib_path) {
        let size = metadata.len() as f64 / 1024.0 / 1024.0;
        println!("  {}: {:.2} MB", target.name, size);
      }
    }
  }

  Ok(())
}
