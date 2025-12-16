pub mod formatter;
pub mod workspace;

use serde::{Deserialize, Serialize};

/// 配置文件名(不含扩展名)
pub const CONFIG_FILE_NAMES: &[&str] = &["novelsaga.config", ".novelsaga"];

/// 支持的配置文件扩展名
pub const CONFIG_FILE_EXTENSIONS_BASE: &[&str] = &["json", "json5", "yaml", "toml", "ron", "corn"];

pub const CONFIG_FILE_EXTENSIONS_EXT_JS: &[&str] = &["js", "cjs", "mjs"];
pub const CONFIG_FILE_EXTENSIONS_EXT_TS: &[&str] = &["ts", "cts", "mts"];

pub trait Config {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[serde(default)]
pub struct OverridableConfig {
  pub fmt: formatter::FormatConfig,
}

impl Config for OverridableConfig {}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, Copy)]
pub struct RootConfig {
  pub workspace: Option<workspace::WorkspaceConfig>,
}

impl Config for RootConfig {}

#[derive(Debug, Clone, Default, Serialize, Deserialize, Copy)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[serde(default)]
pub struct NovelSagaConfig {
  #[serde(flatten)]
  pub root: Option<RootConfig>,

  #[serde(flatten)]
  pub overridable: OverridableConfig,
}

impl Config for NovelSagaConfig {}
