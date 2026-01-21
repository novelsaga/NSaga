use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "_config.ts")]
#[serde(default)]
pub struct WorkspaceConfig {
  pub cache_dir: String,
  pub novelsaga_dir: String,
}

impl Default for WorkspaceConfig {
  fn default() -> Self {
    let novelsaga_dir = ".novelsaga".to_string();
    let cache_dir = format!("{}/cache", &novelsaga_dir);
    Self {
      cache_dir,
      novelsaga_dir,
    }
  }
}
