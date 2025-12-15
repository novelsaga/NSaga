use novelsaga_core::{article::Article, library::formatter::format_text};

use crate::config::NovelSagaConfig;

pub fn formatter(config: &NovelSagaConfig, content: &str) -> String {
  let article = Article::new(content.to_string());
  let result_article = format_text(&article, &config.fmt);
  result_article.content_ref().to_string()
}
