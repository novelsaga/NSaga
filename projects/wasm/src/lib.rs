use novelsaga_core::{article::Article, config::formatter::FormatConfig, library::formatter::format_text};
use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// 初始化 panic hook,在控制台显示错误
#[wasm_bindgen]
pub fn init_panic_hook() {
  console_error_panic_hook::set_once();
}

/// WASM 导出的格式化函数
#[must_use]
#[wasm_bindgen]
pub fn format_text_wasm(article: &Article, config: &FormatConfig) -> Article {
  format_text(article, config)
}

/// 简化版本 - 使用默认配置
#[must_use]
#[wasm_bindgen]
pub fn format_text_simple(article: &Article) -> Article {
  let config = FormatConfig::default();
  format_text(article, &config)
}
