//! 传输层抽象
//!
//! 定义 Transport trait 和具体实现（stdio）

use crate::bridge::{
  error::Result,
  rpc::types::{RpcRequest, RpcResponse},
};

/// 传输层抽象接口
///
/// 负责 JSON-RPC 消息的发送和接收
pub trait Transport: Send {
  /// 发送 RPC 请求
  ///
  /// # Errors
  ///
  /// 传输层错误
  fn send(&mut self, message: &RpcRequest) -> Result<()>;

  /// 接收 RPC 响应（阻塞直到有数据或出错）
  ///
  /// # Errors
  ///
  /// 传输层错误或 JSON 解析错误
  fn receive(&mut self) -> Result<RpcResponse>;

  /// 关闭传输层
  ///
  /// # Errors
  ///
  /// 关闭失败时返回错误
  fn close(&mut self) -> Result<()>;

  /// 检查传输层是否已关闭
  fn is_closed(&self) -> bool;
}

pub mod stdio;

#[cfg(test)]
mod tests;
