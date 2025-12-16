use novelsaga_core::{article::Article, config::NovelSagaConfig, library::formatter::format_text};

pub fn formatter(config: &NovelSagaConfig, content: &str) -> String {
  let article = Article::new(content.to_string());
  let result_article = format_text(&article, &config.overridable.fmt);
  result_article.content_ref().to_string()
}
