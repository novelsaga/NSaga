use std::{collections::HashMap, error::Error};

use config::{FileFormat, FileStoredFormat, Format};
use gray_matter::{Matter, Pod};

use crate::state::init::Initializer;

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
        // 通过全局单例获取 loader
        let state = Initializer::get().map_err(|_| "Global state not initialized. Call Initializer::init() first.")?;

        let loader = state.feature().js_loader().ok_or("JavaScript loader not available")?;

        let map = loader(text)?;
        let json_str = serde_json::to_string(&map).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        FileFormat::Json.parse(uri, &json_str)
      }
      NovelSagaFileFormat::TypeScript => {
        // 通过全局单例获取 loader
        let state = Initializer::get().map_err(|_| "Global state not initialized. Call Initializer::init() first.")?;

        let loader = state.feature().ts_loader().ok_or("TypeScript loader not available")?;

        let map = loader(text)?;
        let json_str = serde_json::to_string(&map).map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        FileFormat::Json.parse(uri, &json_str)
      }
    }
  }
}
