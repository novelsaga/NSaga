use serde::{Deserialize, Serialize};

/// 格式化配置
/// 用于测试config用
///
/// **导出目标**: WASM, SO
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
#[cfg_attr(feature = "ffi", safer_ffi::derive_ReprC)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[repr(C)]
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
