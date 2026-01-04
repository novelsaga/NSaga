use serde::{Deserialize, Serialize};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkspaceConfig {
  #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
  pub cache_dir: String,
  #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
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
