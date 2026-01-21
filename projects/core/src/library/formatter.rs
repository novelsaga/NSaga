use crate::{article, config};

/// 内部格式化函数 - 纯字符串处理逻辑
///
/// 供 diplomat FFI 和内部使用
#[must_use]
pub fn format_text_internal(content: &str, indent_spaces: usize, blank_lines_between_paragraphs: usize) -> String {
  let pangu_formatted = pangu::spacing(content).into_owned();
  let lines: Vec<&str> = pangu_formatted.lines().collect();

  // 丢弃只有空格的行
  let mut formatted_lines: Vec<String> = Vec::new();
  for line in lines {
    if line.trim().is_empty() {
      continue;
    }
    let indented_line = format!("{}{}", " ".repeat(indent_spaces), line.trim());
    formatted_lines.push(indented_line);
  }

  // 在段落之间添加空行
  let mut result_lines: Vec<String> = Vec::new();
  for (i, line) in formatted_lines.iter().enumerate() {
    result_lines.push(line.clone());
    if i < formatted_lines.len() - 1 {
      for _ in 0..blank_lines_between_paragraphs {
        result_lines.push(String::new());
      }
    }
  }

  result_lines.join("\n")
}

/// 核心格式化函数 - 纯计算逻辑,无 I/O
#[must_use]
pub fn format_text(article: &article::Article, config: &config::formatter::FormatConfig) -> article::Article {
  let formatted = format_text_internal(
    article.content_ref(),
    config.indent_spaces,
    config.blank_lines_between_paragraphs,
  );
  article::Article::new(formatted)
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
