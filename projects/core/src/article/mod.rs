// Article 类型统一使用 String 以符合 Cargo 的 feature unification 规则
// 在 workspace 中,当 wasm 包启用 wasm feature 时,所有包都会看到启用了该 feature
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Article {
  content: String,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl Article {
  #[must_use]
  #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
  pub fn new(content: String) -> Self {
    Self { content }
  }

  #[must_use]
  #[wasm_bindgen::prelude::wasm_bindgen(getter)]
  pub fn content(&self) -> String {
    self.content.clone()
  }
}

impl Article {
  /// 从 &str 创建 Article (会克隆字符串)
  #[cfg(not(feature = "wasm"))]
  #[must_use]
  pub fn new(content: impl Into<String>) -> Self {
    Self {
      content: content.into(),
    }
  }

  #[must_use]
  pub fn content_ref(&self) -> &str {
    &self.content
  }
}
