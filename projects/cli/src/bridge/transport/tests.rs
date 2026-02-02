//! Transport 层集成测试
//!
//! 这些测试需要真实的 `config-bridge` 进程，因此标记为 `#[ignore]`
//! 手动运行: `cargo test --package novelsaga-cli bridge::transport::tests -- --nocapture --ignored`

#[cfg(test)]
mod integration_tests {
  use std::{
    fs,
    process::{Command, Stdio},
  };

  use serde_json::json;
  use tempfile::tempdir;

  use super::super::{Transport, stdio::StdioTransport};
  use crate::{
    assets::AssetLocator,
    bridge::{
      env_keys,
      error::BridgeError,
      rpc::types::{RpcRequest, RpcResponse},
    },
  };

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_stdio_transport_with_config_bridge() {
    use std::fs;

    use tempfile::tempdir;

    // 启动 config-bridge 子进程
    // 使用环境变量获取 workspace 根目录

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

    println!("Using bridge script: {}", bridge_script.display());
    println!("Using config: {}", config_path.display());

    let mut child = Command::new("node")
      .arg(&bridge_script)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit()) // 让 stderr 输出到控制台
      .env("NSAGA_CLI_CONFIG_PATH", &config_path)
      .env("NSAGA_RUNTIME", "node") // 明确指定使用 Node.js runtime
      .spawn()
      .expect("Failed to spawn config-bridge process");

    // 给子进程一些启动时间
    std::thread::sleep(std::time::Duration::from_millis(200));

    // 创建传输层
    let mut transport = StdioTransport::from_child(&mut child).expect("Failed to create transport");

    // 发送 JSON-RPC 请求
    let request = RpcRequest::new(json!(1), "config.get", json!({}));

    println!("Sending request: {request:?}");
    transport.send(&request).expect("Failed to send request");

    // 接收响应
    println!("Waiting for response...");
    let response = transport.receive().expect("Failed to receive response");

    println!("Received response: {response:?}");

    // 验证响应
    assert!(response.is_success() || response.is_error());
    assert_eq!(response.id(), &json!(1));

    if let RpcResponse::Success(success) = response {
      println!("Success result: {:#}", success.result);
      // 验证配置包含 workspace 字段
      assert!(success.result.get("workspace").is_some());
    }

    // 清理
    transport.close().expect("Failed to close transport");
    child.kill().expect("Failed to kill child");
    let _ = child.wait();
    println!("✅ Test passed!");
  }

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_stdio_transport_multiple_requests() {
    use std::fs;

    use tempfile::tempdir;

    let dir = tempdir().expect("Failed to create temp dir");
    let config_path = dir.path().join("test-config.mjs");

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

    let mut child = Command::new("node")
      .arg(&bridge_script)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .env("NSAGA_CLI_CONFIG_PATH", &config_path)
      .spawn()
      .expect("Failed to spawn");

    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut transport = StdioTransport::from_child(&mut child).expect("Failed to create transport");

    // 发送多个请求
    for i in 1..=3 {
      let request = RpcRequest::new(json!(i), "config.get", json!({}));
      transport.send(&request).expect("Failed to send");

      let response = transport.receive().expect("Failed to receive");
      assert_eq!(response.id(), &json!(i));
      println!("Request {i} completed");
    }

    transport.close().expect("Failed to close");
    child.kill().expect("Failed to kill child");
    let _ = child.wait();
  }

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_stdio_transport_error_response() {
    let bridge_script = AssetLocator::locate("js/dist/config-bridge.js").expect("Failed to locate config-bridge.js");

    let mut child = Command::new("node")
      .arg(&bridge_script)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      // 故意不设置 CONFIG_PATH，触发错误
      .spawn()
      .expect("Failed to spawn");

    std::thread::sleep(std::time::Duration::from_millis(100));
    let mut transport = StdioTransport::from_child(&mut child).expect("Failed to create transport");

    let request = RpcRequest::new(json!(1), "config.get", json!({}));
    transport.send(&request).expect("Failed to send");

    let response = transport.receive().expect("Failed to receive");

    // 应该是错误响应
    assert!(response.is_error());
    println!("Error response: {response:?}");

    transport.close().expect("Failed to close");
    child.kill().expect("Failed to kill child");
    let _ = child.wait();
  }

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_stdio_transport_timeout() {
    // 测试超时机制：创建一个不响应的进程

    let _bridge_script = AssetLocator::locate("js/dist/config-bridge.js").ok();

    // 启动 bridge 但不设置配置，它会等待输入
    let mut child = Command::new("sleep")
      .arg("10") // 使用 sleep 命令模拟无响应
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()
      .expect("Failed to spawn");

    std::thread::sleep(std::time::Duration::from_millis(100));

    // 创建传输层，设置 2 秒超时
    let mut transport = StdioTransport::from_child(&mut child)
      .expect("Failed to create transport")
      .with_timeout(std::time::Duration::from_secs(2));

    let request = RpcRequest::new(json!(1), "config.get", json!({}));
    let _ = transport.send(&request); // 发送可能失败，因为 sleep 不读 stdin

    println!("Waiting for response (should timeout)...");
    let start = std::time::Instant::now();
    let result = transport.receive();
    let elapsed = start.elapsed();

    println!("Elapsed: {elapsed:?}");

    // 应该超时
    assert!(result.is_err());
    assert!(elapsed >= std::time::Duration::from_secs(2));
    assert!(elapsed < std::time::Duration::from_secs(3));

    if let Err(e) = result {
      println!("Error (expected): {e:?}");
      // 应该是超时错误
      assert!(matches!(e, BridgeError::Timeout(_)));
    }

    println!("✅ Timeout test passed");

    transport.close().expect("Failed to close");
    child.kill().expect("Failed to kill child");
    let _ = child.wait();
  }
}
