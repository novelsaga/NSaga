// Article 类型统一使用 String 以符合 Cargo 的 feature unification 规则
// 在 workspace 中,当为 wasm32 架构时启用 wasm_bindgen 相关导出
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
#[cfg_attr(feature = "ffi", safer_ffi::derive_ReprC)]
#[cfg_attr(feature = "ffi", repr(opaque))]
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Article {
  content: String,
}

#[cfg(feature = "ffi")]
use safer_ffi::prelude::*;

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
impl Article {
  /// 获取内容引用（内部使用，不导出到 WASM）
  #[must_use]
  pub fn content_ref(&self) -> &str {
    &self.content
  }
}

#[cfg(not(target_arch = "wasm32"))]
impl Article {
  /// 从 &str 创建 Article (会克隆字符串)
  #[must_use]
  pub fn new(content: impl Into<String>) -> Self {
    Self {
      content: content.into(),
    }
  }

  /// 获取内容引用
  #[must_use]
  pub fn content_ref(&self) -> &str {
    &self.content
  }

  /// 获取内容副本（为了与 wasm32 架构统一，会克隆字符串）
  #[must_use]
  pub fn content(&self) -> String {
    self.content.clone()
  }
}

#[cfg(feature = "ffi")]
#[ffi_export]
pub fn novelsaga_article_new(content: char_p::Ref<'_>) -> repr_c::Box<Article> {
  Box::new(Article::new(content.to_str())).into()
}

#[cfg(feature = "ffi")]
#[ffi_export]
pub fn novelsaga_article_get_content(article: &Article) -> char_p::Box {
  article.content().try_into().unwrap()
}
