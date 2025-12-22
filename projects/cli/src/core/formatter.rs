use novelsaga_core::{article::Article, config::formatter::FormatConfig, library::formatter::format_text};

pub fn formatter(config: &FormatConfig, content: &str) -> String {
  let article = Article::new(content.to_string());
  let result_article = format_text(&article, config);
  result_article.content_ref().to_string()
}
