//! Bridge 模块：负责 Rust 与 JavaScript 之间的 JSON-RPC 通信
//!
//! ## 架构
//!
//! - `error`: 统一错误类型定义
//! - `rpc`: JSON-RPC 2.0 协议层（类型定义和客户端）
//! - `runtime`: JavaScript 运行时检测和管理
//! - `transport`: 传输层抽象（stdin/stdout 实现）
//! - `manager`: Bridge 管理层（Bridge trait 和 ``BridgeManager``）

#![allow(dead_code)]
#![allow(unused_imports)]

pub mod env_keys;
pub mod error;
pub mod manager;
pub mod rpc;
pub mod runtime;
pub mod transport;

// Re-exports for convenience
pub use error::{BridgeError, Result};
pub use manager::{BridgeManager, bridge::Bridge, config_bridge::ConfigBridge};
pub use rpc::{
  client::RpcClient,
  types::{RpcRequest, RpcResponse, error_codes},
};
pub use runtime::{RuntimeDiscovery, RuntimeInfo, RuntimeProcess, RuntimeType};
pub use transport::Transport;
