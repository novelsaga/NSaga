use novelsaga_core::format_text;

use crate::config::NovelSagaConfig;

pub fn formatter(config: &NovelSagaConfig, content: &str) -> String {
  format_text(content, &config.fmt)
}
