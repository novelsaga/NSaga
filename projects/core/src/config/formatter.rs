use serde::{Deserialize, Serialize};

/// 格式化配置
/// 用于测试config用
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
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
