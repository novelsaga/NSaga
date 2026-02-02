//! JSON-RPC 2.0 类型定义
//!
//! 参考规范: <https://www.jsonrpc.org/specification>

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcRequest {
  /// 协议版本，必须是 "2.0"
  pub jsonrpc: String,
  /// 请求 ID（字符串或数字）
  pub id: Value,
  /// 方法名（如 "config.get"）
  pub method: String,
  /// 参数（对象或数组）
  pub params: Value,
}

impl RpcRequest {
  /// 创建新的 RPC 请求
  #[must_use]
  pub fn new(id: Value, method: impl Into<String>, params: Value) -> Self {
    Self {
      jsonrpc: "2.0".to_string(),
      id,
      method: method.into(),
      params,
    }
  }

  /// 序列化为 JSON 字符串
  ///
  /// # Errors
  ///
  /// 如果序列化失败返回错误
  pub fn to_json_string(&self) -> serde_json::Result<String> {
    serde_json::to_string(self)
  }
}

/// JSON-RPC 2.0 响应（成功或错误）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcResponse {
  /// 成功响应
  Success(RpcSuccessResponse),
  /// 错误响应
  Error(RpcErrorResponse),
}

impl RpcResponse {
  /// 是否为成功响应
  #[must_use]
  pub fn is_success(&self) -> bool {
    matches!(self, Self::Success(_))
  }

  /// 是否为错误响应
  #[must_use]
  pub fn is_error(&self) -> bool {
    matches!(self, Self::Error(_))
  }

  /// 获取响应 ID
  #[must_use]
  pub fn id(&self) -> &Value {
    match self {
      Self::Success(r) => &r.id,
      Self::Error(r) => &r.id,
    }
  }

  /// 从 JSON 字符串解析
  ///
  /// # Errors
  ///
  /// 如果解析失败返回错误
  pub fn from_json_str(s: &str) -> serde_json::Result<Self> {
    serde_json::from_str(s)
  }
}

/// JSON-RPC 2.0 成功响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcSuccessResponse {
  /// 协议版本
  pub jsonrpc: String,
  /// 请求 ID
  pub id: Value,
  /// 结果数据
  pub result: Value,
}

/// JSON-RPC 2.0 错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcErrorResponse {
  /// 协议版本
  pub jsonrpc: String,
  /// 请求 ID
  pub id: Value,
  /// 错误对象
  pub error: RpcError,
}

/// JSON-RPC 2.0 错误对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
  /// 错误码
  pub code: i32,
  /// 错误消息
  pub message: String,
  /// 附加数据（可选）
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<Value>,
}

/// JSON-RPC 2.0 通知（无需响应）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcNotification {
  /// 协议版本
  pub jsonrpc: String,
  /// 方法名
  pub method: String,
  /// 参数
  pub params: Value,
}

/// JSON-RPC 2.0 标准错误码
pub mod error_codes {
  /// 解析错误（无效的 JSON）
  pub const PARSE_ERROR: i32 = -32700;
  /// 无效的请求（不符合 JSON-RPC 规范）
  pub const INVALID_REQUEST: i32 = -32600;
  /// 方法不存在
  pub const METHOD_NOT_FOUND: i32 = -32601;
  /// 无效的参数
  pub const INVALID_PARAMS: i32 = -32602;
  /// 内部错误
  pub const INTERNAL_ERROR: i32 = -32603;

  // 自定义错误码范围 (-32000 到 -32099)
  /// 服务不存在
  pub const SERVICE_NOT_FOUND: i32 = -32001;
  /// 服务执行错误
  pub const SERVICE_ERROR: i32 = -32002;
  /// 超时错误
  pub const TIMEOUT_ERROR: i32 = -32003;
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_rpc_request_creation() {
    let req = RpcRequest::new(json!(1), "config.get", json!({}));
    assert_eq!(req.jsonrpc, "2.0");
    assert_eq!(req.id, json!(1));
    assert_eq!(req.method, "config.get");
  }

  #[test]
  fn test_rpc_request_serialization() {
    let req = RpcRequest::new(json!(1), "config.get", json!({}));
    let json_str = req.to_json_string().unwrap();
    assert!(json_str.contains("\"jsonrpc\":\"2.0\""));
    assert!(json_str.contains("\"method\":\"config.get\""));
  }

  #[test]
  fn test_parse_success_response() {
    let json = r#"{"jsonrpc":"2.0","id":1,"result":{"foo":"bar"}}"#;
    let response = RpcResponse::from_json_str(json).unwrap();
    assert!(response.is_success());
    assert_eq!(response.id(), &json!(1));
  }

  #[test]
  fn test_parse_error_response() {
    let json = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32600,"message":"Invalid Request"}}"#;
    let response = RpcResponse::from_json_str(json).unwrap();
    assert!(response.is_error());
    if let RpcResponse::Error(err_resp) = response {
      assert_eq!(err_resp.error.code, error_codes::INVALID_REQUEST);
      assert_eq!(err_resp.error.message, "Invalid Request");
    }
  }
}
