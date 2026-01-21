use std::{
  env,
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::{Context, Result};

/// Get project root directory
pub fn project_root() -> PathBuf {
  Path::new(&env!("CARGO_MANIFEST_DIR"))
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}

/// Get output directory for WASM builds
pub fn wasm_out_dir() -> PathBuf {
  env::var("WASM_OUT_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| project_root().join("out/wasm"))
}

/// Get output directory for SO builds
pub fn so_out_dir() -> PathBuf {
  env::var("SO_OUT_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| project_root().join("out/so"))
}

/// Get output directory for CLI builds
pub fn cli_out_dir() -> PathBuf {
  env::var("CLI_OUT_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| project_root().join("out/cli"))
}

/// Get core project directory
pub fn core_project_dir() -> PathBuf {
  env::var("CORE_PROJECT_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| project_root().join("projects/core"))
}

/// Get CLI project directory
pub fn cli_project_dir() -> PathBuf {
  env::var("CLI_PROJECT_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| project_root().join("projects/cli"))
}

/// Get target directory
pub fn target_dir() -> PathBuf {
  env::var("CARGO_TARGET_DIR")
    .map(PathBuf::from)
    .unwrap_or_else(|_| project_root().join("target"))
}

/// Run a command and return error if it fails
pub fn run_command(cmd: &mut Command) -> Result<()> {
  let status = cmd
    .status()
    .with_context(|| format!("Failed to execute command: {:?}", cmd))?;

  if !status.success() {
    anyhow::bail!("Command failed with status: {}", status);
  }

  Ok(())
}

/// Detect current platform
pub fn detect_platform() -> Result<String> {
  let os = env::consts::OS;
  let arch = env::consts::ARCH;

  match (os, arch) {
    ("linux", "x86_64") => Ok("linux-x64".to_string()),
    ("linux", "aarch64") => Ok("linux-arm64".to_string()),
    ("macos", "x86_64") => Ok("macos-x64".to_string()),
    ("macos", "aarch64") => Ok("macos-arm64".to_string()),
    _ => anyhow::bail!("Unsupported platform: {}-{}", os, arch),
  }
}
