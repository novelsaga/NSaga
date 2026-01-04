//! WASM 导出模块
//!
//! 此模块包含所有 WASM 相关的导出功能，只在 wasm32 架构下编译。

use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

/// 初始化 panic hook，在浏览器控制台显示错误
#[wasm_bindgen]
pub fn init_panic_hook() {
  console_error_panic_hook::set_once();
}
