//! Bridge 统一错误类型定义

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

  /// IO 错误
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),

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

  /// Bridge 未找到
  #[error("Bridge not found: {0}")]
  BridgeNotFound(String),

  /// 运行时未找到
  #[error("Runtime not found: {0}")]
  RuntimeNotFound(String),

  /// 其他错误
  #[error("Unknown error: {0}")]
  Other(String),
}

/// Bridge 模块的 Result 类型别名
pub type Result<T> = std::result::Result<T, BridgeError>;
