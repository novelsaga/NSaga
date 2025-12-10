use novelsaga_core::{FormatConfig, format_text};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// 初始化 panic hook,在控制台显示错误
#[wasm_bindgen]
pub fn init_panic_hook() {
  console_error_panic_hook::set_once();
}

/// WASM 格式化配置
#[wasm_bindgen]
#[derive(Default)]
pub struct WasmFormatConfig {
  inner: FormatConfig,
}

#[wasm_bindgen]
impl WasmFormatConfig {
  #[must_use]
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self {
      inner: FormatConfig::default(),
    }
  }

  #[must_use]
  #[wasm_bindgen(getter)]
  pub fn indent_spaces(&self) -> usize {
    self.inner.indent_spaces
  }

  #[wasm_bindgen(setter)]
  pub fn set_indent_spaces(&mut self, value: usize) {
    self.inner.indent_spaces = value;
  }

  #[must_use]
  #[wasm_bindgen(getter)]
  pub fn blank_lines_between_paragraphs(&self) -> usize {
    self.inner.blank_lines_between_paragraphs
  }

  #[wasm_bindgen(setter)]
  pub fn set_blank_lines_between_paragraphs(&mut self, value: usize) {
    self.inner.blank_lines_between_paragraphs = value;
  }
}

/// WASM 导出的格式化函数
#[must_use]
#[wasm_bindgen]
pub fn format_text_wasm(content: &str, config: &WasmFormatConfig) -> String {
  format_text(content, &config.inner)
}

/// 简化版本 - 使用默认配置
#[must_use]
#[wasm_bindgen]
pub fn format_text_simple(content: &str) -> String {
  let config = FormatConfig::default();
  format_text(content, &config)
}
