#[cfg(feature = "ffi")]
use safer_ffi::prelude::*;

use crate::{article, config};

/// 核心格式化函数 - 纯计算逻辑,无 I/O
#[must_use]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn format_text(article: &article::Article, config: &config::formatter::FormatConfig) -> article::Article {
  let pangu_formatted = pangu::spacing(article.content_ref()).into_owned();
  let lines: Vec<&str> = pangu_formatted.lines().collect();

  // 丢弃只有空格的行
  let mut formatted_lines: Vec<String> = Vec::new();
  for line in lines {
    if line.trim().is_empty() {
      continue;
    }
    let indented_line = format!("{}{}", " ".repeat(config.indent_spaces), line.trim());
    formatted_lines.push(indented_line);
  }

  // 在段落之间添加空行
  let mut result_lines: Vec<String> = Vec::new();
  for (i, line) in formatted_lines.iter().enumerate() {
    result_lines.push(line.clone());
    if i < formatted_lines.len() - 1 {
      for _ in 0..config.blank_lines_between_paragraphs {
        result_lines.push(String::new());
      }
    }
  }

  article::Article::new(result_lines.join("\n"))
}

/// FFI 版本的格式化函数
///
/// **导出目标**: SO
#[cfg(feature = "ffi")]
#[ffi_export]
pub fn novelsaga_format_text(
  article: &article::Article,
  config: &config::formatter::FormatConfig,
) -> repr_c::Box<article::Article> {
  Box::new(format_text(article, config)).into()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format_text() {
    let config = config::formatter::FormatConfig::default();
    let input = "你好world\n\n这是test";
    let article = article::Article::new(input.to_string());
    let result = format_text(&article, &config);
    #[cfg(target_arch = "wasm32")]
    let result_str = result.content();
    #[cfg(not(target_arch = "wasm32"))]
    let result_str = result.content_ref();
    assert!(result_str.contains("你好 world")); // pangu 应该在中英文之间添加空格
    assert!(result_str.contains("这是 test"));
  }
}
