use std::process::Command;

use anyhow::Result;

use super::utils::*;

pub fn generate_ts() -> Result<()> {
  println!("TS-RS: Generating TypeScript bindings...");

  let project = project_root();
  let export_dir = project.join("projects/cli/assets/js/src/js-config-bridge/src/types");

  // Ensure directory exists
  std::fs::create_dir_all(&export_dir)?;

  // Set export directory
  std::env::set_var(
    "TS_RS_EXPORT_DIR",
    "../../projects/cli/assets/js/src/js-config-bridge/src/types",
  );

  // Run test to trigger export
  run_command(
    Command::new("cargo")
      .args([
        "test",
        "-p",
        "novelsaga-core",
        "--lib",
        "config::tests::export_bindings",
      ])
      .current_dir(&project),
  )?;

  // Add License header
  let target_file = export_dir.join("_config.ts");
  if target_file.exists() {
    let content = std::fs::read_to_string(&target_file)?;
    let with_header = format!("// Licensed under the MIT License.\n{}", content);
    std::fs::write(&target_file, with_header)?;
    println!("✅ Generated {} with MIT header", target_file.display());
  }

  Ok(())
}
