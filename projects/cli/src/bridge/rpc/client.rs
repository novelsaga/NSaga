//! JSON-RPC 客户端实现

use std::time::Duration;

use serde_json::{Value, json};

use super::types::{RpcErrorResponse, RpcRequest, RpcResponse, RpcSuccessResponse};
use crate::bridge::{
  error::{BridgeError, Result},
  transport::Transport,
};

/// RPC 客户端
///
/// 提供同步的 JSON-RPC 方法调用接口
pub struct RpcClient {
  transport: Box<dyn Transport>,
  timeout: Duration,
  next_id: u64,
}

impl RpcClient {
  /// 创建新的 RPC 客户端
  #[must_use]
  pub fn new(transport: Box<dyn Transport>) -> Self {
    Self {
      transport,
      timeout: Duration::from_secs(30),
      next_id: 1,
    }
  }

  /// 设置请求超时时间
  #[must_use]
  pub fn with_timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    self
  }

  /// 生成下一个请求 ID（递增）
  fn next_request_id(&mut self) -> Value {
    let id = self.next_id;
    self.next_id += 1;
    json!(id)
  }

  /// 同步调用 RPC 方法
  ///
  /// # Arguments
  ///
  /// * `method` - 方法名（如 "config.get"）
  /// * `params` - 参数（JSON 对象或数组）
  ///
  /// # Errors
  ///
  /// - 传输层错误
  /// - JSON 解析错误
  /// - RPC 协议错误
  /// - 响应 ID 不匹配
  pub fn call(&mut self, method: &str, params: Value) -> Result<Value> {
    let id = self.next_request_id();
    let request = RpcRequest::new(id.clone(), method, params);

    // 发送请求
    self.transport.send(&request)?;

    // 接收响应（带超时）
    let response = self.receive_with_timeout()?;

    // 验证响应 ID
    if response.id() != &id {
      return Err(BridgeError::InvalidResponseId {
        expected: id,
        actual: response.id().clone(),
      });
    }

    // 处理响应
    match response {
      RpcResponse::Success(RpcSuccessResponse { result, .. }) => Ok(result),
      RpcResponse::Error(RpcErrorResponse { error, .. }) => Err(BridgeError::RpcError {
        code: error.code,
        message: error.message,
        data: error.data,
      }),
    }
  }

  /// 接收响应（带超时）
  ///
  /// TODO: 当前为阻塞版本，未来可改进为真正的超时机制
  fn receive_with_timeout(&mut self) -> Result<RpcResponse> {
    self.transport.receive()
  }

  /// 关闭客户端
  ///
  /// # Errors
  ///
  /// 传输层关闭失败时返回错误
  pub fn close(mut self) -> Result<()> {
    self.transport.close()
  }
}

#[cfg(test)]
mod tests {
  // 测试将在后续添加，需要 mock transport
}
