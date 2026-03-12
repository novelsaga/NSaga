use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};

use anyhow::{Context, Result};
use tempfile::tempdir;

#[derive(Debug, Clone, Copy)]
enum Runtime {
  Node,
  Bun,
  Deno,
}

impl Runtime {
  fn executable(&self) -> &'static str {
    match self {
      Runtime::Node => "node",
      Runtime::Bun => "bun",
      Runtime::Deno => "deno",
    }
  }

  fn display_name(&self) -> &'static str {
    match self {
      Runtime::Node => "Node.js",
      Runtime::Bun => "Bun",
      Runtime::Deno => "Deno",
    }
  }

  fn is_available(&self) -> bool {
    Command::new(self.executable()).arg("--version").output().is_ok()
  }

  fn all_available() -> Vec<Runtime> {
    vec![Runtime::Node, Runtime::Bun, Runtime::Deno]
      .into_iter()
      .filter(|r| r.is_available())
      .collect()
  }
}

#[derive(Debug, Clone, Copy)]
enum ConfigType {
  JsEsm, // .mjs
  JsCjs, // .cjs
  TsEsm, // .mts
  TsCjs, // .cts
}

impl ConfigType {
  fn extension(&self) -> &'static str {
    match self {
      ConfigType::JsEsm => "mjs",
      ConfigType::JsCjs => "cjs",
      ConfigType::TsEsm => "mts",
      ConfigType::TsCjs => "cts",
    }
  }

  fn display_name(&self) -> &'static str {
    match self {
      ConfigType::JsEsm => "JS ESM (.mjs)",
      ConfigType::JsCjs => "JS CJS (.cjs)",
      ConfigType::TsEsm => "TS ESM (.mts)",
      ConfigType::TsCjs => "TS CJS (.cts)",
    }
  }

  fn content(&self) -> &'static str {
    match self {
      ConfigType::JsEsm => {
        r#"export default {
  workspace: {
    cache_dir: ".cache/novelsaga"
  },
  fmt: {
    indent_spaces: 2,
    blank_lines_between_paragraphs: 2
  }
};"#
      }
      ConfigType::JsCjs => {
        r#"module.exports = {
  workspace: {
    cache_dir: ".cache/novelsaga"
  },
  fmt: {
    indent_spaces: 2,
    blank_lines_between_paragraphs: 2
  }
};"#
      }
      ConfigType::TsEsm => {
        r#"// TypeScript ESM with type annotations
interface WorkspaceConfig {
  cache_dir: string;
}

interface FmtConfig {
  indent_spaces: number;
  blank_lines_between_paragraphs: number;
}

interface NovelSagaConfig {
  workspace: WorkspaceConfig;
  fmt: FmtConfig;
}

const config: NovelSagaConfig = {
  workspace: {
    cache_dir: ".cache/novelsaga"
  },
  fmt: {
    indent_spaces: 2,
    blank_lines_between_paragraphs: 2
  }
};

export default config;"#
      }
      ConfigType::TsCjs => {
        r#"// TypeScript CommonJS with type annotations
interface WorkspaceConfig {
  cache_dir: string;
}

interface FmtConfig {
  indent_spaces: number;
  blank_lines_between_paragraphs: number;
}

interface NovelSagaConfig {
  workspace: WorkspaceConfig;
  fmt: FmtConfig;
}

const config: NovelSagaConfig = {
  workspace: {
    cache_dir: ".cache/novelsaga"
  },
  fmt: {
    indent_spaces: 2,
    blank_lines_between_paragraphs: 2
  }
};

module.exports = config;"#
      }
    }
  }

  fn requires_typescript(&self) -> bool {
    matches!(self, ConfigType::TsEsm | ConfigType::TsCjs)
  }
}

/// Run E2E tests for CLI
pub fn run_e2e_tests() -> Result<()> {
  println!("🧪 Running CLI E2E tests...\n");

  // 1. 确保 CLI 已构建
  ensure_cli_built()?;

  // 2. 确保 JS bridges 已构建（使用统一的构建系统）
  crate::tasks::build::build_all()?;

  // 3. 检测可用的运行时
  let available_runtimes = Runtime::all_available();
  println!("🔍 Available runtimes:");
  for runtime in &available_runtimes {
    println!("   ✓ {}", runtime.display_name());
  }
  if available_runtimes.is_empty() {
    anyhow::bail!("No JavaScript runtimes available (node/bun/deno)");
  }
  println!();

  // 4. 运行测试场景
  let mut passed = 0;
  let mut failed = 0;
  let mut skipped = 0;

  // 基础测试（不依赖特定运行时）
  run_test("Config Priority", test_config_priority, &mut passed, &mut failed);
  run_test("Error Handling", test_error_scenarios, &mut passed, &mut failed);

  // 矩阵测试：运行时 × 配置类型
  let config_types = vec![
    ConfigType::JsEsm,
    ConfigType::JsCjs,
    ConfigType::TsEsm,
    ConfigType::TsCjs,
  ];

  for runtime in &available_runtimes {
    for config_type in &config_types {
      // Node.js < 23.6 不原生支持 TypeScript
      if config_type.requires_typescript() && matches!(runtime, Runtime::Node) && !is_node_ts_supported() {
        let test_name = format!("{} + {}", runtime.display_name(), config_type.display_name());
        print!("📝 Test: {} ... ", test_name);
        println!("⊘ (Skipped: Node.js < 23.6)");
        skipped += 1;
        continue;
      }

      let test_name = format!("{} + {}", runtime.display_name(), config_type.display_name());
      run_test(
        &test_name,
        || test_runtime_config(*runtime, *config_type),
        &mut passed,
        &mut failed,
      );
    }
  }

  // 5. 输出测试结果
  println!("\n{}", "=".repeat(60));
  println!("📊 Test Results:");
  println!("   ✅ Passed: {}", passed);
  if skipped > 0 {
    println!("   ⊘ Skipped: {}", skipped);
  }
  if failed > 0 {
    println!("   ❌ Failed: {}", failed);
    anyhow::bail!("{} test(s) failed", failed);
  }
  println!("{}", "=".repeat(60));
  println!("\n✅ All E2E tests passed!");

  Ok(())
}

fn run_test<F>(name: &str, test_fn: F, passed: &mut i32, failed: &mut i32)
where
  F: FnOnce() -> Result<()>,
{
  print!("📝 Test: {} ... ", name);
  match test_fn() {
    Ok(_) => {
      println!("✅");
      *passed += 1;
    }
    Err(e) => {
      println!("❌");
      eprintln!("   Error: {}", e);
      *failed += 1;
    }
  }
}

fn ensure_cli_built() -> Result<()> {
  println!("🔨 Building CLI...");
  let status = Command::new("cargo")
    .args(["build", "-p", "novelsaga-cli"])
    .status()
    .context("Failed to run cargo build")?;

  if !status.success() {
    anyhow::bail!("Failed to build CLI");
  }
  println!("   ✅ CLI built successfully\n");
  Ok(())
}

/// 通用的运行时 + 配置类型测试
fn test_runtime_config(runtime: Runtime, config_type: ConfigType) -> Result<()> {
  let dir = tempdir()?;
  let config_name = format!("novelsaga.config.{}", config_type.extension());
  let config_path = dir.path().join(&config_name);

  fs::write(&config_path, config_type.content())?;

  let cli_bin = get_cli_binary()?;

  // 设置环境变量强制使用指定的运行时
  let output = Command::new(&cli_bin)
    .arg("--help")
    .current_dir(dir.path())
    .env("NSAGA_CLI_RUNTIME", runtime.executable())
    .output()
    .context("Failed to run CLI")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!("CLI failed with {}: {}", runtime.display_name(), stderr);
  }

  Ok(())
}

fn test_config_priority() -> Result<()> {
  let dir = tempdir()?;

  // 创建多个配置文件，验证优先级
  // novelsaga.config.mjs 应该优先于 .json
  fs::write(
    dir.path().join("novelsaga.config.json"),
    r#"{"workspace":{"cache_dir":"wrong"}}"#,
  )?;

  fs::write(
    dir.path().join("novelsaga.config.mjs"),
    r#"export default {workspace:{cache_dir:".cache/novelsaga"}};"#,
  )?;

  let cli_bin = get_cli_binary()?;
  let output = Command::new(&cli_bin)
    .arg("--help")
    .current_dir(dir.path())
    .output()
    .context("Failed to run CLI")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!("CLI failed: {}", stderr);
  }

  // TODO: 实际验证加载的是 .mjs 而非 .json
  // 目前只能验证 CLI 没有 crash

  Ok(())
}

fn test_error_scenarios() -> Result<()> {
  let dir = tempdir()?;

  // 测试语法错误
  fs::write(
    dir.path().join("novelsaga.config.mjs"),
    "export default { invalid syntax",
  )?;

  let cli_bin = get_cli_binary()?;
  let output = Command::new(&cli_bin)
    .arg("--help")
    .current_dir(dir.path())
    .output()
    .context("Failed to run CLI")?;

  // 应该优雅地处理错误（使用默认配置），而不是 panic
  // 当前实现：CLI 应该能启动，只是配置加载失败
  if !output.status.success() {
    // 检查是否是预期的错误
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.contains("config") && !stderr.contains("syntax") {
      anyhow::bail!("Unexpected error: {}", stderr);
    }
  }

  Ok(())
}

// Helper functions

fn get_workspace_root() -> Result<PathBuf> {
  let output = Command::new("cargo")
    .args(["locate-project", "--workspace", "--message-format=plain"])
    .output()
    .context("Failed to locate workspace")?;

  if !output.status.success() {
    anyhow::bail!("Failed to get workspace root");
  }

  let cargo_toml = String::from_utf8(output.stdout)?.trim().to_string();
  let path = Path::new(&cargo_toml).parent().context("Invalid workspace path")?;

  Ok(path.to_path_buf())
}

fn get_cli_binary() -> Result<PathBuf> {
  let workspace_root = get_workspace_root()?;
  let cli_bin = workspace_root.join("target/debug/novelsaga");

  if !cli_bin.exists() {
    anyhow::bail!("CLI binary not found at {:?}", cli_bin);
  }

  Ok(cli_bin)
}

fn is_node_ts_supported() -> bool {
  if let Ok(version) = get_node_version() {
    version.0 > 23 || (version.0 == 23 && version.1 >= 6)
  } else {
    false
  }
}

fn get_node_version() -> Result<(u32, u32, u32)> {
  let output = Command::new("node")
    .arg("--version")
    .output()
    .context("Failed to get Node.js version")?;

  if !output.status.success() {
    anyhow::bail!("Failed to get Node.js version");
  }

  let version_str = String::from_utf8(output.stdout)?;
  let version_str = version_str.trim().trim_start_matches('v');

  let parts: Vec<u32> = version_str.split('.').take(3).map(|s| s.parse().unwrap_or(0)).collect();

  Ok((
    parts.first().copied().unwrap_or(0),
    parts.get(1).copied().unwrap_or(0),
    parts.get(2).copied().unwrap_or(0),
  ))
}
