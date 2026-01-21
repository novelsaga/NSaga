//! Diplomat 绑定生成任务

use std::process::Command;

use anyhow::Result;

use super::utils::*;

/// 生成 C++ 绑定
pub fn generate_cpp() -> Result<()> {
  println!("正在生成 C++ 绑定...");

  let root = project_root();
  let core_path = root.join("projects/core");
  let out_dir = root.join("target/diplomat/cpp");

  // 确保输出目录存在
  std::fs::create_dir_all(&out_dir)?;

  // 使用 diplomat-tool 库 API 生成 C++ 绑定
  let entry_path = core_path.join("src/lib.rs");
  diplomat_tool::r#gen(
    &entry_path,
    "cpp",
    &out_dir,
    &diplomat_tool::DocsUrlGenerator::default(),
    diplomat_tool::config::Config::default(),
    false, // silent
  )?;

  println!("✓ C++ 绑定已生成到: {}", out_dir.display());
  Ok(())
}

/// 生成 TypeScript (WASM) 绑定
pub fn generate_typescript() -> Result<()> {
  println!("正在生成 TypeScript 绑定...");

  let root = project_root();
  let core_path = root.join("projects/core");
  let target_dir = target_dir();
  let out_dir = root.join("target/diplomat/ts");

  // 确保输出目录存在
  std::fs::create_dir_all(&out_dir)?;

  // 首先需要构建 WASM
  println!("📦 构建 WASM...");
  run_command(
    Command::new("cargo")
      .args([
        "build",
        "--release",
        "--target",
        "wasm32-unknown-unknown",
        "--target-dir",
        target_dir.to_str().unwrap(),
      ])
      .current_dir(&core_path),
  )?;

  let wasm_file = target_dir
    .join("wasm32-unknown-unknown/release")
    .join("novelsaga_core.wasm");

  if !wasm_file.exists() {
    anyhow::bail!("WASM 文件未找到: {}", wasm_file.display());
  }

  // 使用 diplomat-tool 库 API 生成 TypeScript 绑定
  let entry_path = core_path.join("src/lib.rs");
  let config = diplomat_tool::config::Config::default();

  diplomat_tool::r#gen(
    &entry_path,
    "js",
    &out_dir,
    &diplomat_tool::DocsUrlGenerator::default(),
    config,
    false, // silent
  )?;

  // 生成 diplomat.config.mjs 文件，包含 wasm_path
  let config_content = r#"export default {
    wasm_path: new URL('./novelsaga_core.wasm', import.meta.url),
};
"#;
  std::fs::write(out_dir.join("diplomat.config.mjs"), config_content)?;

  // 复制 WASM 文件到输出目录
  std::fs::copy(&wasm_file, out_dir.join("novelsaga_core.wasm"))?;

  println!("✓ TypeScript 绑定已生成到: {}", out_dir.display());
  Ok(())
}

/// 生成所有绑定
pub fn generate_all() -> Result<()> {
  generate_cpp()?;
  generate_typescript()?;
  println!("✓ 所有 Diplomat 绑定生成完成");
  Ok(())
}
