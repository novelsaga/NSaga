use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export_to = "_workspace_config.ts")]
#[serde(default)]
pub struct WorkspaceConfig {
  pub cache_dir: String,
  pub novelsaga_dir: String,
}

impl Default for WorkspaceConfig {
  fn default() -> Self {
    let novelsaga_dir = ".novelsaga".to_string();
    let cache_dir = ".cache/novelsaga".to_string();
    Self {
      cache_dir,
      novelsaga_dir,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_default_cache_dir_is_xdg_compliant() {
    let config = WorkspaceConfig::default();
    assert_eq!(config.cache_dir, ".cache/novelsaga");
  }

  #[test]
  fn test_default_novelsaga_dir() {
    let config = WorkspaceConfig::default();
    assert_eq!(config.novelsaga_dir, ".novelsaga");
  }

  #[test]
  fn test_default_structure() {
    let config = WorkspaceConfig::default();
    assert_eq!(config.cache_dir, ".cache/novelsaga");
    assert_eq!(config.novelsaga_dir, ".novelsaga");
  }
}
