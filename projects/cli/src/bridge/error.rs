//! Bridge 统一错误类型定义

use std::path::PathBuf;

use thiserror::Error;

/// Bridge 模块的统一错误类型
#[derive(Debug, Error)]
pub enum BridgeError {
  /// JSON 解析错误
  #[error("JSON parse error: {0}")]
  JsonParseError(#[from] serde_json::Error),

  /// JSON-RPC 协议错误
  #[error("RPC error: code={code}, message={message}")]
  RpcError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
  },

  /// IO 错误（带操作上下文）
  #[error("{context}\n\nCause: {source}")]
  IoError { context: String, source: std::io::Error },

  /// 传输层已关闭
  #[error("Transport closed")]
  TransportClosed,

  /// 请求超时
  #[error("Request timeout after {0:?}")]
  Timeout(std::time::Duration),

  /// 响应 ID 不匹配
  #[error("Invalid response: expected id {expected}, got {actual}")]
  InvalidResponseId {
    expected: serde_json::Value,
    actual: serde_json::Value,
  },

  /// Bridge 未初始化
  #[error("Bridge not initialized")]
  NotInitialized,

  /// Bridge not found (with suggestion)
  #[error("Bridge not found: {name}\n\n{suggestion}")]
  BridgeNotFound { name: String, suggestion: String },

  /// Runtime not found (with searched paths and suggestion)
  #[error("Runtime not found: {runtime_type}\n\nSearched paths:\n{searched_paths}\n\nSuggestion:\n{suggestion}")]
  RuntimeNotFound {
    runtime_type: String,
    searched_paths: String,
    suggestion: String,
  },

  /// 其他错误
  #[error("{0}")]
  Other(String),
}

impl From<std::io::Error> for BridgeError {
  fn from(err: std::io::Error) -> Self {
    BridgeError::IoError {
      context: "IO operation failed".to_string(),
      source: err,
    }
  }
}

/// Bridge 模块的 Result 类型别名
pub type Result<T> = std::result::Result<T, BridgeError>;
