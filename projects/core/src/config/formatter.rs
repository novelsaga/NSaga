use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 格式化配置
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "_config.ts")]
#[serde(default)]
pub struct FormatConfig {
  pub indent_spaces: usize,
  pub blank_lines_between_paragraphs: usize,
}

impl Default for FormatConfig {
  fn default() -> Self {
    Self {
      indent_spaces: 4,
      blank_lines_between_paragraphs: 1,
    }
  }
}
