use novelsaga_core::config::formatter::FormatConfig;
use serde::{Deserialize, Serialize};

/// 配置文件名(不含扩展名)
pub const CONFIG_FILE_NAMES: &[&str] = &["novelsaga.config", ".novelsaga"];

/// 支持的配置文件扩展名
pub const CONFIG_FILE_EXTENSIONS: &[&str] = &["toml", "yaml", "json", "json5", "ron", "corn"];

/// `NovelSaga` LSP 服务器的主配置结构
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct NovelSagaConfig {
  /// 格式化配置
  pub fmt: FormatConfig,
}
