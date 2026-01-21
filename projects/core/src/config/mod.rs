pub mod file_def;
pub mod fileformat;
pub mod formatter;
pub mod workspace;
// pub mod manager;

use merge_struct::merge;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub trait Config {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "_config.ts")]
#[serde(default)]
pub struct OverridableConfig {
  pub fmt: formatter::FormatConfig,
}

impl Config for OverridableConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, Default, TS)]
#[ts(export, export_to = "_config.ts")]
#[serde(default)]
pub struct RootConfig {
  pub workspace: Option<workspace::WorkspaceConfig>,
}

impl RootConfig {
  #[must_use]
  pub fn to_novelsaga_config(&self) -> NovelSagaConfig {
    NovelSagaConfig {
      root: self.clone(),
      overridable: OverridableConfig::default(),
    }
  }
}

impl Config for RootConfig {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "_config.ts", rename = "_NovelSagaConfig")]
#[serde(default)]
pub struct NovelSagaConfig {
  #[serde(flatten)]
  pub root: RootConfig,

  #[serde(flatten)]
  pub overridable: OverridableConfig,
}

impl NovelSagaConfig {
  #[must_use]
  pub fn from_root_and_override(root: Option<RootConfig>, overridable: Option<OverridableConfig>) -> Self {
    Self {
      root: root.unwrap_or_default(),
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn export_bindings() {
    formatter::FormatConfig::export().expect("failed to export FormatConfig");
    workspace::WorkspaceConfig::export().expect("failed to export WorkspaceConfig");
    RootConfig::export().expect("failed to export RootConfig");
    OverridableConfig::export().expect("failed to export OverridableConfig");
    NovelSagaConfig::export().expect("failed to export NovelSagaConfig");
  }
}
