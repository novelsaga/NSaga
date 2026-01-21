use std::{fs, process::Command};

use anyhow::Result;

use super::utils::*;

pub fn build() -> Result<()> {
  println!("🔨 Building NovelSaga WASM from core...");

  let wasm_out = wasm_out_dir();
  let core_dir = core_project_dir();
  let target_dir = target_dir();

  // Clean and create output directory
  if wasm_out.exists() {
    fs::remove_dir_all(&wasm_out)?;
  }
  fs::create_dir_all(&wasm_out)?;

  // Step 1: Build with cargo for wasm32-unknown-unknown target
  println!("📦 Building with cargo for wasm32-unknown-unknown...");
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
      .current_dir(&core_dir),
  )?;

  // Step 2: Use diplomat-tool library API to generate TypeScript/WASM bindings
  let wasm_file = target_dir
    .join("wasm32-unknown-unknown/release")
    .join("novelsaga_core.wasm");

  if !wasm_file.exists() {
    anyhow::bail!("WASM file not found at: {}", wasm_file.display());
  }

  println!("\n⚙️  Generating TypeScript bindings with diplomat-tool...");

  // 使用库 API 生成绑定
  let entry_path = core_dir.join("src/lib.rs");
  let config = diplomat_tool::config::Config::default();

  diplomat_tool::r#gen(
    &entry_path,
    "js",
    &wasm_out,
    &diplomat_tool::DocsUrlGenerator::default(),
    config,
    false, // not silent
  )?;

  // 生成 diplomat.config.mjs 文件
  let config_content = r#"export default {
    wasm_path: new URL('./novelsaga_core.wasm', import.meta.url),
};
"#;
  fs::write(wasm_out.join("diplomat.config.mjs"), config_content)?;

  // 复制 WASM 文件到输出目录
  let wasm_dest = wasm_out.join("novelsaga_core.wasm");
  fs::copy(&wasm_file, &wasm_dest)?;

  println!("\n✅ Build complete!");

  // Show package size
  if let Ok(metadata) = fs::metadata(&wasm_dest) {
    let size = metadata.len() as f64 / 1024.0 / 1024.0;
    println!("📊 WASM size: {:.2} MB", size);
  }

  // Create package.json for npm compatibility
  let package_json = serde_json::json!({
    "name": "novelsaga-core",
    "version": "0.1.0",
    "type": "module",
    "files": [
      "novelsaga_core.wasm",
      "*.mjs",
      "*.d.ts",
      "diplomat.config.mjs"
    ],
    "main": "index.mjs",
    "types": "index.d.ts"
  });

  fs::write(
    wasm_out.join("package.json"),
    serde_json::to_string_pretty(&package_json)?,
  )?;

  println!("🚀 WASM package ready at: {}", wasm_out.display());

  Ok(())
}
