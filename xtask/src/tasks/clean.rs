use std::{fs, process::Command};

use anyhow::Result;

use super::utils::*;

pub fn clean_wasm() -> Result<()> {
  let wasm_out = wasm_out_dir();
  if wasm_out.exists() {
    fs::remove_dir_all(&wasm_out)?;
  }
  println!("✅ WASM artifacts cleaned");
  Ok(())
}

pub fn clean_so() -> Result<()> {
  let so_out = so_out_dir();
  if so_out.exists() {
    fs::remove_dir_all(&so_out)?;
  }
  println!("✅ Shared Library artifacts cleaned");
  Ok(())
}

pub fn clean_cli() -> Result<()> {
  let cli_out = cli_out_dir();
  if cli_out.exists() {
    fs::remove_dir_all(&cli_out)?;
  }
  println!("✅ CLI artifacts cleaned");
  Ok(())
}

pub fn clean_all() -> Result<()> {
  clean_wasm()?;
  clean_so()?;
  clean_cli()?;

  // Run cargo clean
  Command::new("cargo")
    .arg("clean")
    .current_dir(project_root())
    .status()?;

  // Remove out directory
  let out_dir = project_root().join("out");
  if out_dir.exists() {
    fs::remove_dir_all(&out_dir)?;
  }

  println!("✅ All artifacts cleaned");
  Ok(())
}
