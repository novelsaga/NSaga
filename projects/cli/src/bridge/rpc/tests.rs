//! RPC 层集成测试
//!
//! 这些测试需要真实的 `config-bridge` 进程，因此标记为 `#[ignore]`
//! 手动运行: `cargo test --package novelsaga-cli bridge::rpc::tests -- --nocapture --ignored`

#[cfg(test)]
mod integration_tests {
  use std::{
    fs,
    process::{Command, Stdio},
    time::Duration,
  };

  use serde_json::json;
  use tempfile::tempdir;

  use crate::{
    assets::AssetLocator,
    bridge::{env_keys, rpc::client::RpcClient, transport::stdio::StdioTransport},
  };

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_rpc_client_config_get() {
    use std::fs;

    use tempfile::tempdir;

    // AssetLocator will find the bridge script
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

    // 启动 config-bridge
    let mut child = Command::new("node")
      .arg(&bridge_script)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .env(env_keys::CONFIG_PATH, &config_path)
      .env(env_keys::RUNTIME, "node") // 明确指定使用 Node.js runtime
      .spawn()
      .expect("Failed to spawn config-bridge");

    std::thread::sleep(Duration::from_millis(100));

    // 创建传输层和 RPC 客户端
    let transport = StdioTransport::from_child(&mut child).expect("Failed to create transport");
    let mut client = RpcClient::new(Box::new(transport));

    // 调用 config.get
    println!("Calling config.get...");
    let result = client.call("config.get", json!({})).expect("RPC call failed");

    println!("Result: {result:#}");

    // 验证结果
    assert!(result.is_object());
    assert!(result.get("workspace").is_some());

    let workspace = result.get("workspace").unwrap();
    assert!(workspace.get("cache_dir").is_some());

    println!("✅ config.get test passed");

    // 清理
    client.close().expect("Failed to close client");
    child.kill().expect("Failed to kill child");
    let _ = child.wait();
  }

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_rpc_client_multiple_calls() {
    use std::fs;

    use tempfile::tempdir;

    // AssetLocator will find the bridge script
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
      .env(env_keys::CONFIG_PATH, &config_path)
      .env(env_keys::RUNTIME, "node") // 明确指定使用 Node.js runtime
      .spawn()
      .expect("Failed to spawn");

    std::thread::sleep(Duration::from_millis(100));
    let transport = StdioTransport::from_child(&mut child).expect("Failed to create transport");
    let mut client = RpcClient::new(Box::new(transport));

    // 多次调用
    for i in 1..=5 {
      println!("Call #{i}");
      let result = client.call("config.get", json!({})).expect("Call failed");
      assert!(result.get("workspace").is_some());
    }

    println!("✅ Multiple calls test passed");

    client.close().expect("Failed to close");
    child.kill().expect("Failed to kill child");
    let _ = child.wait();
  }

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_rpc_client_with_timeout() {
    use std::fs;

    use tempfile::tempdir;

    // AssetLocator will find the bridge script
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
      .env(env_keys::CONFIG_PATH, &config_path)
      .spawn()
      .expect("Failed to spawn");

    std::thread::sleep(Duration::from_millis(100));
    let transport = StdioTransport::from_child(&mut child).expect("Failed to create transport");
    let mut client = RpcClient::new(Box::new(transport)).with_timeout(Duration::from_secs(5));

    let result = client.call("config.get", json!({})).expect("Call failed");
    assert!(result.get("workspace").is_some());

    println!("✅ Timeout test passed");

    client.close().expect("Failed to close");
    child.kill().expect("Failed to kill child");
    let _ = child.wait();
  }

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_rpc_client_error_handling() {
    // AssetLocator will find the bridge script
    let bridge_script = AssetLocator::locate("js/dist/config-bridge.js").expect("Failed to locate config-bridge.js");

    // 不设置 CONFIG_PATH，触发错误
    let mut child = Command::new("node")
      .arg(&bridge_script)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()
      .expect("Failed to spawn");

    std::thread::sleep(Duration::from_millis(100));
    let transport = StdioTransport::from_child(&mut child).expect("Failed to create transport");
    let mut client = RpcClient::new(Box::new(transport));

    let result = client.call("config.get", json!({}));

    // 应该返回 RPC 错误
    assert!(result.is_err());
    let err = result.unwrap_err();
    println!("Error: {err:?}");

    println!("✅ Error handling test passed");

    client.close().expect("Failed to close");
    child.kill().expect("Failed to kill child");
    let _ = child.wait();
  }

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_rpc_client_with_function_config() {
    use std::fs;

    use tempfile::tempdir;

    // AssetLocator will find the bridge script
    let dir = tempdir().expect("Failed to create temp dir");
    let config_path = dir.path().join("test-config-fn.mjs");

    // 函数式配置
    fs::write(
      &config_path,
      r#"export default (settings) => ({
  workspace: {
    cache_dir: ".novelsaga/cache",
    novelsaga_dir: ".novelsaga"
  },
  fmt: {
    indent: 2,
    line_width: 100
  }
});"#,
    )
    .expect("Failed to write config");

    let bridge_script = AssetLocator::locate("js/dist/config-bridge.js").expect("Failed to locate config-bridge.js");

    let mut child = Command::new("node")
      .arg(&bridge_script)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .env(env_keys::CONFIG_PATH, &config_path)
      .env(env_keys::PROJECT_ROOT, "/test/project")
      .spawn()
      .expect("Failed to spawn");

    std::thread::sleep(Duration::from_millis(100));
    let transport = StdioTransport::from_child(&mut child).expect("Failed to create transport");
    let mut client = RpcClient::new(Box::new(transport));

    let result = client.call("config.get", json!({})).expect("Call failed");
    println!("Function config result: {result:#}");

    assert!(result.get("workspace").is_some());

    println!("✅ Function config test passed");

    client.close().expect("Failed to close");
    child.kill().expect("Failed to kill child");
    let _ = child.wait();
  }
}
