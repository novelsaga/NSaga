use std::{collections::HashMap, error::Error};

use config::{FileStoredFormat, Format};
use gray_matter::Matter;
use serde_yaml::Value as YamlValue;

#[derive(Debug, Clone, Copy)]
pub enum NovelSagaFileFormat {
  Markdown,
}

impl FileStoredFormat for NovelSagaFileFormat {
  fn file_extensions(&self) -> &'static [&'static str] {
    match self {
      NovelSagaFileFormat::Markdown => &["md", "markdown"],
    }
  }
}

impl Format for NovelSagaFileFormat {
  fn parse(
    &self,
    _uri: Option<&String>,
    text: &str,
  ) -> Result<HashMap<String, config::Value>, Box<dyn Error + Send + Sync>> {
    match self {
      NovelSagaFileFormat::Markdown => {
        use gray_matter::engine::YAML;
        let matter = Matter::<YAML>::new();
        let result = matter.parse::<YamlValue>(text);
        match result {
          Ok(parsed) => {
            if let Some(front_matter) = parsed.data {
              // 将 YamlValue 转换为 HashMap<String, config::Value>
              yaml_value_to_config_map(&front_matter)
            } else {
              Ok(HashMap::new())
            }
          }
          Err(err) => Err(Box::new(err) as Box<dyn Error + Send + Sync>),
        }
      }
    }
  }
}

fn yaml_value_to_config_map(yaml: &YamlValue) -> Result<HashMap<String, config::Value>, Box<dyn Error + Send + Sync>> {
  match yaml {
    YamlValue::Mapping(map) => {
      let mut result = HashMap::new();
      for (key, value) in map {
        if let YamlValue::String(key_str) = key {
          result.insert(key_str.clone(), yaml_value_to_config_value(value)?);
        }
      }
      Ok(result)
    }
    _ => Ok(HashMap::new()),
  }
}

fn yaml_value_to_config_value(yaml: &YamlValue) -> Result<config::Value, Box<dyn Error + Send + Sync>> {
  let value = match yaml {
    YamlValue::Null => config::Value::new(None, config::ValueKind::Nil),
    YamlValue::Bool(b) => config::Value::new(None, config::ValueKind::Boolean(*b)),
    YamlValue::Number(n) => {
      if let Some(i) = n.as_i64() {
        config::Value::new(None, config::ValueKind::I64(i))
      } else if let Some(f) = n.as_f64() {
        config::Value::new(None, config::ValueKind::Float(f))
      } else {
        config::Value::new(None, config::ValueKind::Nil)
      }
    }
    YamlValue::String(s) => config::Value::new(None, config::ValueKind::String(s.clone())),
    YamlValue::Sequence(seq) => {
      let mut arr = Vec::new();
      for item in seq {
        arr.push(yaml_value_to_config_value(item)?);
      }
      config::Value::new(None, config::ValueKind::Array(arr))
    }
    YamlValue::Mapping(map) => {
      let mut table = HashMap::new();
      for (key, value) in map {
        if let YamlValue::String(key_str) = key {
          table.insert(key_str.clone(), yaml_value_to_config_value(value)?);
        }
      }
      config::Value::new(None, config::ValueKind::Table(table))
    }
    YamlValue::Tagged(tagged) => return yaml_value_to_config_value(&tagged.value),
  };
  Ok(value)
}
