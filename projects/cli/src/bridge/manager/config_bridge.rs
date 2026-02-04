//! `ConfigBridge` 实现
//!
//! 负责与 config-bridge.js 通信，加载配置文件

use std::{cell::RefCell, collections::HashMap, path::Path};

use serde_json::Value;

use super::bridge::Bridge;
use crate::bridge::{
  env_keys,
  error::{BridgeError, Result},
  rpc::client::RpcClient,
  runtime::{RuntimeInfo, process::RuntimeProcess},
  transport::stdio::StdioTransport,
};

/// `ConfigBridge` 实现
///
/// 通过 JSON-RPC 与 config-bridge.js 通信
pub struct ConfigBridge {
  /// Bridge 名称
  name: String,
  /// RPC 客户端 (注意：必须在 process 之前 drop，以确保管道先关闭)
  client: RpcClient,
  /// 运行时进程（使用 ``RefCell`` 实现内部可变性）
  process: RefCell<RuntimeProcess>,
}

impl ConfigBridge {
  /// 创建新的 `ConfigBridge`
  ///
  /// # Arguments
  ///
  /// * `runtime_info` - JavaScript 运行时信息
  /// * `bridge_script` - config-bridge.js 脚本路径
  /// * `env` - 环境变量（包含 ``NSAGA_CLI_``* 等）
  ///
  /// # Errors
  ///
  /// 启动进程或创建传输层失败
  pub fn new(runtime_info: RuntimeInfo, bridge_script: &Path, env: HashMap<String, String>) -> Result<Self> {
    // 启动运行时进程
    let mut process = RuntimeProcess::spawn(runtime_info, bridge_script, env)?;

    // 从进程中 take IO 句柄
    let (stdin, stdout) = process.take_io().ok_or_else(|| {
      BridgeError::Other(
        "无法从运行时进程获取 IO\n\n原因: stdin/stdout 可能已被取出\n\n解决方案: 检查 RuntimeProcess 的使用状态"
          .to_string(),
      )
    })?;

    // 创建传输层
    let transport = StdioTransport::from_buffered(stdin, stdout)?;

    // 创建 RPC 客户端
    let client = RpcClient::new(Box::new(transport));

    Ok(Self {
      name: "config-bridge".to_string(),
      process: RefCell::new(process),
      client,
    })
  }
}

impl Bridge for ConfigBridge {
  fn name(&self) -> &str {
    &self.name
  }

  fn call(&mut self, method: &str, params: Value) -> Result<Value> {
    self.client.call(method, params)
  }

  fn is_healthy(&self) -> bool {
    // 使用 RefCell 借用进程来检查状态
    self.process.borrow_mut().is_alive()
  }

  fn shutdown(&mut self) -> Result<()> {
    // 等待一小段时间让进程自然退出
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 如果进程还活着，强制 kill
    if self.process.borrow_mut().is_alive() {
      self.process.borrow_mut().kill()?;
      let _ = self.process.borrow_mut().wait();
    }

    Ok(())
  }
}

impl Drop for ConfigBridge {
  fn drop(&mut self) {
    // 不再自动调用 shutdown()，因为：
    // 1. shutdown() 应该由 BridgeManager 显式调用
    // 2. 避免双重清理导致的问题
    // 进程会在 RuntimeProcess drop 时自动清理
  }
}

#[cfg(test)]
mod tests {
  use std::fs;

  use serde_json::json;
  use tempfile::tempdir;

  use super::*;
  use crate::{
    assets::AssetLocator,
    bridge::runtime::{RuntimeType, discovery::RuntimeDiscovery},
  };

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_config_bridge_new_and_call() {
    let dir = tempdir().expect("Failed to create temp dir");
    let config_path = dir.path().join("test-config.mjs");

    // 动态生成测试配置
    fs::write(
      &config_path,
      r#"export default {
  workspace: {
    cache_dir: ".novelsaga/cache",
    novelsaga_dir: ".novelsaga"
  },
  fmt: {
    indent: 2,
    line_width: 100
  }
};"#,
    )
    .expect("Failed to write config");

    let bridge_script = AssetLocator::locate("js/dist/config-bridge.js").expect("Failed to locate config-bridge.js");

    // 检测 Node.js 运行时
    let discovery = RuntimeDiscovery::new();
    let runtime_info = discovery
      .find_runtime(RuntimeType::NodeJs)
      .expect("Node.js not found")
      .expect("Node.js runtime not available");

    // 准备环境变量
    let mut env = HashMap::new();
    env.insert(
      env_keys::CONFIG_PATH.to_string(),
      config_path.to_string_lossy().to_string(),
    );
    env.insert(env_keys::RUNTIME.to_string(), "node".to_string());

    // 创建 ConfigBridge
    let mut bridge = ConfigBridge::new(runtime_info, &bridge_script, env).expect("Failed to create ConfigBridge");

    // 检查健康状态
    assert!(bridge.is_healthy());

    // 调用 config.get
    let result = bridge.call("config.get", json!({})).expect("Failed to call config.get");

    println!("Config result: {result:#}");
    assert!(result.is_object());
    assert!(result.get("workspace").is_some());

    // 关闭 Bridge
    bridge.shutdown().expect("Failed to shutdown bridge");

    // 检查进程已停止
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert!(!bridge.is_healthy());

    println!("✅ ConfigBridge test passed");
  }
}
