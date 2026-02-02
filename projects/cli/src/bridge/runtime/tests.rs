//! 运行时管理集成测试
//!
//! 手动运行: cargo test --package novelsaga-cli bridge::runtime::tests -- --nocapture --ignored

#[cfg(test)]
mod integration_tests {
  use std::{
    collections::HashMap,
    fs,
    io::{BufRead, Write},
  };

  use tempfile::tempdir;

  use crate::{
    assets::AssetLocator,
    bridge::{
      env_keys,
      runtime::{RuntimeDiscovery, RuntimeProcess, RuntimeType},
    },
  };

  #[test]
  #[ignore] // 需要实际运行时环境
  fn test_find_nodejs() {
    let discovery = RuntimeDiscovery::new();
    let result = discovery.find_runtime(RuntimeType::NodeJs).expect("Discovery failed");

    if let Some(info) = result {
      println!("Found Node.js:");
      println!("  Path: {:?}", info.path);
      println!("  Version: {}", info.version);
      println!("  Supports TS: {}", info.supports_native_typescript());
      assert!(info.version.starts_with('v'));
    } else {
      println!("Node.js not found (this is OK if not installed)");
    }
  }

  #[test]
  #[ignore]
  fn test_find_all_runtimes() {
    let discovery = RuntimeDiscovery::new();
    let runtimes = discovery.find_all_runtimes().expect("Discovery failed");

    println!("Found {} runtime(s):", runtimes.len());
    for info in runtimes {
      println!("  - {:?}: {} at {:?}", info.runtime_type, info.version, info.path);
    }
  }

  #[test]
  #[ignore]
  fn test_find_best_runtime() {
    let discovery = RuntimeDiscovery::new();
    let result = discovery.find_best_runtime();

    match result {
      Ok(info) => {
        println!("Best runtime:");
        println!("  Type: {:?}", info.runtime_type);
        println!("  Path: {:?}", info.path);
        println!("  Version: {}", info.version);
      }
      Err(e) => {
        println!("No runtime found: {}", e);
        // 如果系统没有安装任何运行时，测试应该失败
        panic!("Expected at least one runtime to be installed");
      }
    }
  }

  #[test]
  #[ignore]
  fn test_spawn_simple_process() {
    use std::collections::HashMap;

    use crate::bridge::runtime::RuntimeProcess;

    let discovery = RuntimeDiscovery::new();
    let runtime = discovery.find_best_runtime().expect("No runtime found");

    println!("Using runtime: {:?} {}", runtime.runtime_type, runtime.version);

    // 创建一个简单的测试脚本
    let test_script = std::env::temp_dir().join("test_simple.js");
    std::fs::write(&test_script, "console.log('Hello from test'); process.exit(0);")
      .expect("Failed to write test script");

    let env = HashMap::new();
    let mut process = RuntimeProcess::spawn(runtime, &test_script, env).expect("Failed to spawn process");

    println!("Process spawned with PID: {:?}", process.pid());

    // 等待进程结束
    std::thread::sleep(std::time::Duration::from_millis(500));

    let is_running = process.is_running();
    println!("Process running after 500ms: {}", is_running);

    // 清理
    process.kill().ok();
    std::fs::remove_file(test_script).ok();
  }

  #[test]
  #[ignore]
  fn test_process_with_config_bridge() {
    use std::{
      collections::HashMap,
      io::{BufRead, Write},
    };

    use crate::bridge::runtime::RuntimeProcess;

    let discovery = RuntimeDiscovery::new();
    let runtime = discovery.find_best_runtime().expect("No runtime found");

    // 获取 config-bridge.js 路径

    let bridge_script = AssetLocator::locate("js/dist/config-bridge.js").expect("Failed to locate config-bridge.js");

    // 动态生成测试配置
    let dir = tempfile::tempdir().expect("Failed to create temp dir");
    let test_config = dir.path().join("test-config.mjs");
    std::fs::write(
      &test_config,
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

    if !bridge_script.exists() {
      println!("Skipping test: bridge script not found at {:?}", bridge_script);
      return;
    }

    let mut env = HashMap::new();
    env.insert(
      env_keys::CONFIG_PATH.to_string(),
      test_config.to_string_lossy().to_string(),
    );

    let mut process = RuntimeProcess::spawn(runtime, &bridge_script, env).expect("Failed to spawn bridge");

    println!("Bridge started with PID: {:?}", process.pid());

    // 等待 bridge 启动
    std::thread::sleep(std::time::Duration::from_millis(200));

    // 发送 JSON-RPC 请求
    let request = r#"{"jsonrpc":"2.0","id":1,"method":"config.get","params":{}}"#;
    println!("Sending request: {}", request);

    process.send(request).expect("Failed to send request");

    // 读取响应
    if let Some(stdout) = process.stdout_mut() {
      let mut response = String::new();
      if let Ok(_) = stdout.read_line(&mut response) {
        println!("Received response: {}", response.trim());

        // 简单验证响应格式
        assert!(response.contains("\"jsonrpc\":\"2.0\""));
        assert!(response.contains("\"id\":1"));
      } else {
        println!("Failed to read response");
      }
    }

    // 清理
    process.shutdown().expect("Failed to shutdown");
    println!("✅ Process with config-bridge test passed");
  }
}
