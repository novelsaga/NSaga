// Article 类型 - 内部实现
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Article {
  content: String,
}

impl Article {
  /// 从字符串创建 Article
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
}
