use std::process::Command;

use anyhow::Result;

use super::utils::*;

pub fn generate_ts() -> Result<()> {
  println!("TS-RS: Generating TypeScript bindings...");

  let project = project_root();
  let export_dir = project.join("projects/cli-js-bridges/config-bridge/src/types");

  // Ensure directory exists
  std::fs::create_dir_all(&export_dir)?;

  // Set export directory
  std::env::set_var(
    "TS_RS_EXPORT_DIR",
    project
      .join("projects/cli-js-bridges/config-bridge/src/types")
      .to_str()
      .unwrap(),
  );

  // Run all export tests to trigger binding generation
  run_command(
    Command::new("cargo")
      .args(["test", "-p", "novelsaga-core", "--lib", "export_bindings"])
      .current_dir(&project),
  )?;

  // Add eslint-disable header to all generated files
  for entry in std::fs::read_dir(&export_dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.extension().map_or(false, |ext| ext == "ts")
      && path
        .file_name()
        .map_or(false, |name| name.to_str().map_or(false, |s| s.starts_with('_')))
    {
      let content = std::fs::read_to_string(&path)?;
      if !content.starts_with("/* eslint-disable */") {
        let with_headers = format!("/* eslint-disable */\n{}", content);
        std::fs::write(&path, with_headers)?;
        println!("✅ Generated {} with headers", path.display());
      }
    }
  }

  Ok(())
}
