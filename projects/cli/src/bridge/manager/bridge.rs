//! Bridge 抽象接口
//!
//! 定义统一的 Bridge 接口，所有具体的 Bridge 实现都需要实现此 trait

use serde_json::Value;

use crate::bridge::error::Result;

/// Bridge 抽象接口
///
/// 所有 Bridge 实现（ConfigBridge、PluginBridge 等）都需要实现此 trait
pub trait Bridge: Send {
  /// 获取 Bridge 名称
  fn name(&self) -> &str;

  /// 调用 Bridge 服务的方法
  ///
  /// # Arguments
  ///
  /// * `method` - 服务方法名（如 "config.get"）
  /// * `params` - 方法参数（JSON Value）
  ///
  /// # Errors
  ///
  /// 通信错误或服务执行错误
  fn call(&mut self, method: &str, params: Value) -> Result<Value>;

  /// 检查 Bridge 是否健康（进程是否存活）
  fn is_healthy(&self) -> bool;

  /// 关闭 Bridge（终止子进程）
  ///
  /// # Errors
  ///
  /// 关闭进程失败
  fn shutdown(&mut self) -> Result<()>;
}
