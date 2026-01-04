use std::{collections::HashMap, error::Error};

use config::{FileFormat, FileStoredFormat, Format};
use gray_matter::{Matter, Pod};

#[derive(Debug, Clone, Copy)]
pub enum NovelSagaFileFormat {
  Markdown,
  JavaScript,
  TypeScript,
}

impl FileStoredFormat for NovelSagaFileFormat {
  fn file_extensions(&self) -> &'static [&'static str] {
    match self {
      NovelSagaFileFormat::Markdown => &["md", "markdown"],
      NovelSagaFileFormat::JavaScript => &["js", "cjs", "mjs"],
      NovelSagaFileFormat::TypeScript => &["ts", "cts", "mts"],
    }
  }
}

impl Format for NovelSagaFileFormat {
  fn parse(
    &self,
    uri: Option<&String>,
    text: &str,
  ) -> Result<HashMap<String, config::Value>, Box<dyn Error + Send + Sync>> {
    match self {
      NovelSagaFileFormat::Markdown => {
        use gray_matter::engine::YAML;
        let matter = Matter::<YAML>::new();
        // 使用 Pod 类型（gray_matter 的默认类型），我们只需要 parsed.matter（原始 YAML 字符串）
        let parsed = matter.parse::<Pod>(text)?;
        FileFormat::Yaml.parse(uri, &parsed.matter)
      }
      NovelSagaFileFormat::JavaScript => {
        // @todo 实现 JS 配置文件的解析
        Err("JavaScript config parsing not implemented".into())
      }
      NovelSagaFileFormat::TypeScript => {
        // @todo 实现 TS 配置文件的解析
        Err("TypeScript config parsing not implemented".into())
      }
    }
  }
}
