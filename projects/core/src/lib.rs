/// 格式化配置
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

/// 核心格式化函数 - 纯计算逻辑,无 I/O
#[must_use]
pub fn format_text(content: &str, config: &FormatConfig) -> String {
  let pangu_formatted = pangu::spacing(content).into_owned();
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

  result_lines.join("\n")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format_text() {
    let config = FormatConfig::default();
    let input = "你好世界\n\n这是测试";
    let result = format_text(input, &config);
    assert!(result.contains("你好 世界")); // pangu 应该添加空格
  }
}
// test comment
