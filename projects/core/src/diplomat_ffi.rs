//! Diplomat FFI 统一入口点
//!
//! 所有 FFI 类型和函数必须在这个单一的 `#[diplomat::bridge]` 模块中定义
//! Diplomat 不支持分散在多个模块中的 FFI 定义

#[allow(clippy::must_use_candidate)]
#[allow(clippy::boxed_local)]
#[allow(clippy::unnecessary_box_returns)]
#[diplomat::bridge]
pub mod ffi {
  use core::fmt::Write;

  use diplomat_runtime::{DiplomatStr, DiplomatWrite};

  // ============ Article 类型 ============

  /// 文章结构
  #[diplomat::opaque]
  pub struct Article(crate::article::Article);

  impl Article {
    /// 创建新文章
    #[diplomat::attr(auto, constructor)]
    pub fn new(content: &DiplomatStr) -> Box<Article> {
      let content_str = core::str::from_utf8(content).unwrap_or("");
      Box::new(Article(crate::article::Article::new(content_str)))
    }

    /// 获取文章内容
    ///
    /// # Panics
    ///
    /// Panics if writing to the ``DiplomatWrite`` buffer fails.
    pub fn get_content(&self, write: &mut DiplomatWrite) {
      write.write_str(self.0.content_ref()).unwrap();
      write.flush();
    }

    /// 获取内容长度
    pub fn len(&self) -> usize {
      self.0.content_ref().len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
      self.0.content_ref().is_empty()
    }

    /// 格式化文章文本
    pub fn format(&self, config: &FormatConfig) -> Box<Article> {
      let formatted = crate::library::formatter::format_text(&self.0, &config.0);
      Box::new(Article(formatted))
    }
  }

  // ============ FormatConfig 类型 ============

  /// 格式化配置
  #[diplomat::opaque]
  pub struct FormatConfig(crate::config::formatter::FormatConfig);

  impl FormatConfig {
    /// 创建默认配置
    #[diplomat::attr(auto, constructor)]
    pub fn new() -> Box<FormatConfig> {
      Box::new(FormatConfig(crate::config::formatter::FormatConfig::default()))
    }

    /// 创建自定义配置
    pub fn with_options(indent_spaces: usize, blank_lines_between_paragraphs: usize) -> Box<FormatConfig> {
      Box::new(FormatConfig(crate::config::formatter::FormatConfig {
        indent_spaces,
        blank_lines_between_paragraphs,
      }))
    }
  }
}
