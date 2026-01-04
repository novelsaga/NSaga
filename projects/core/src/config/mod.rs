pub mod file_def;
pub mod fileformat;
pub mod formatter;
pub mod workspace;
// pub mod manager;

use merge_struct::merge;
use serde::{Deserialize, Serialize};

pub trait Config {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
#[serde(default)]
pub struct OverridableConfig {
  #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
  pub fmt: formatter::FormatConfig,
}

impl Config for OverridableConfig {}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RootConfig {
  #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
  pub workspace: Option<workspace::WorkspaceConfig>,
}

impl RootConfig {
  #[must_use]
  pub fn to_novelsaga_config(&self) -> NovelSagaConfig {
    NovelSagaConfig {
      root: Some(self.clone()),
      overridable: OverridableConfig::default(),
    }
  }
}

impl Config for RootConfig {}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
#[serde(default)]
pub struct NovelSagaConfig {
  #[serde(flatten)]
  #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
  pub root: Option<RootConfig>,

  #[serde(flatten)]
  #[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
  pub overridable: OverridableConfig,
}

impl NovelSagaConfig {
  #[must_use]
  pub fn from_root_and_override(root: Option<RootConfig>, overridable: Option<OverridableConfig>) -> Self {
    Self {
      root,
      overridable: overridable.unwrap_or_default(),
    }
  }

  /// Merge this config with another overridable config.
  ///
  /// # Panics
  ///
  /// Panics if the underlying merge operation fails (propagated from `merge_struct::merge`).
  #[must_use]
  pub fn merge(&self, other: &OverridableConfig) -> Self {
    Self {
      root: self.root.clone(),
      overridable: merge(&self.overridable, other).expect("failed to merge OverridableConfig"),
    }
  }
}

impl Config for NovelSagaConfig {}
