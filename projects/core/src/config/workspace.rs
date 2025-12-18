use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceConfig {
  #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
  pub cache_dir: String,
  #[cfg_attr(feature = "wasm", wasm_bindgen(getter_with_clone))]
  pub novelsaga_dir: String,
  pub respect_git: bool,
}

impl Default for WorkspaceConfig {
  fn default() -> Self {
    let novelsaga_dir = ".novelsaga".to_string();
    let cache_dir = format!("{}/cache", &novelsaga_dir);
    Self {
      cache_dir,
      novelsaga_dir,
      respect_git: true,
    }
  }
}
