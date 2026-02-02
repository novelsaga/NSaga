//! Assets 模块错误类型

use thiserror::Error;

/// Assets 模块的错误类型
#[derive(Debug, Error)]
pub enum AssetError {
  /// 资源文件未找到
  #[error("Asset '{0}' not found. {1}")]
  NotFound(String, String),

  /// IO 错误
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),
}

/// Assets 模块的 Result 类型别名
pub type Result<T> = std::result::Result<T, AssetError>;
