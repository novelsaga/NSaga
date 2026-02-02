//! Bridge 管理器
//!
//! 统一管理多个 Bridge 实例的生命周期

pub mod bridge;
pub mod config_bridge;

use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use serde_json::Value;

use self::bridge::Bridge;
use crate::bridge::{
  env_keys,
  error::{BridgeError, Result},
};

/// Bridge 工厂函数类型
pub type BridgeFactory = Box<dyn Fn() -> Result<Box<dyn Bridge>> + Send>;

/// Bridge 实例的线程安全包装类型
type BridgeInstance = Arc<Mutex<Box<dyn Bridge>>>;

/// Bridge 管理器
///
/// 负责管理多个 Bridge 实例的生命周期
/// - 懒加载：首次调用时才启动 Bridge
/// - 自动重试：失败时自动重试创建
/// - 线程安全：使用 Arc<Mutex<>> 保证并发安全
pub struct BridgeManager {
  bridges: Arc<Mutex<HashMap<String, BridgeInstance>>>,
  factories: Arc<Mutex<HashMap<String, BridgeFactory>>>,
  max_retries: u32,
}

impl BridgeManager {
  /// 创建新的 `BridgeManager`
  #[must_use]
  pub fn new() -> Self {
    Self {
      bridges: Arc::new(Mutex::new(HashMap::new())),
      factories: Arc::new(Mutex::new(HashMap::new())),
      max_retries: 3,
    }
  }

  /// 设置最大重试次数
  #[must_use]
  pub fn with_max_retries(mut self, max_retries: u32) -> Self {
    self.max_retries = max_retries;
    self
  }

  /// 注册 Bridge 工厂函数
  ///
  /// # Arguments
  ///
  /// * `name` - Bridge 名称（如 "config-bridge"）
  /// * `factory` - 创建 Bridge 的工厂函数
  pub fn register<F>(&self, name: impl Into<String>, factory: F)
  where
    F: Fn() -> Result<Box<dyn Bridge>> + Send + 'static,
  {
    let name = name.into();
    let mut factories = self.factories.lock().unwrap();
    factories.insert(name, Box::new(factory));
  }

  /// 调用 Bridge 的方法（懒加载）
  ///
  /// # Arguments
  ///
  /// * `bridge_name` - Bridge 名称
  /// * `method` - 服务方法名
  /// * `params` - 方法参数
  ///
  /// # Errors
  ///
  /// Bridge 未注册、启动失败或调用失败
  pub fn call(&self, bridge_name: &str, method: &str, params: Value) -> Result<Value> {
    // 尝试获取或创建 Bridge
    let bridge_arc = self.get_or_create_bridge(bridge_name)?;

    // 获取锁并调用方法
    let mut bridge = bridge_arc.lock().unwrap();
    bridge.call(method, params)
  }

  /// 获取或创建 Bridge（懒加载 + 重试）
  fn get_or_create_bridge(&self, name: &str) -> Result<BridgeInstance> {
    let mut bridges = self.bridges.lock().unwrap();

    // 如果已存在且健康，直接返回
    if let Some(bridge_arc) = bridges.get(name) {
      // 检查健康状态（持有锁）
      let is_healthy = {
        let bridge = bridge_arc.lock().unwrap();
        bridge.is_healthy()
      };

      if is_healthy {
        return Ok(Arc::clone(bridge_arc));
      }
      // Bridge 不健康，移除它
      bridges.remove(name);
    }

    // 如果不存在，创建新的 Bridge
    let bridge = self.create_bridge_with_retry(name)?;
    let bridge_arc = Arc::new(Mutex::new(bridge));
    bridges.insert(name.to_string(), Arc::clone(&bridge_arc));

    Ok(bridge_arc)
  }

  /// 创建 Bridge（带重试）
  fn create_bridge_with_retry(&self, name: &str) -> Result<Box<dyn Bridge>> {
    let factories = self.factories.lock().unwrap();
    let factory = factories
      .get(name)
      .ok_or_else(|| BridgeError::BridgeNotFound(name.to_string()))?;

    let mut last_error = None;
    for attempt in 1..=self.max_retries {
      match factory() {
        Ok(bridge) => {
          if attempt > 1 {
            eprintln!("[BridgeManager] Bridge '{name}' created on attempt {attempt}");
          }
          return Ok(bridge);
        }
        Err(e) => {
          eprintln!(
            "[BridgeManager] Failed to create bridge '{name}' (attempt {attempt}/{max}): {e}",
            max = self.max_retries
          );
          last_error = Some(e);
          if attempt < self.max_retries {
            std::thread::sleep(std::time::Duration::from_millis(100 * u64::from(attempt)));
          }
        }
      }
    }

    Err(last_error.unwrap())
  }

  /// 检查 Bridge 是否健康
  pub fn is_healthy(&self, name: &str) -> bool {
    let bridges = self.bridges.lock().unwrap();
    if let Some(bridge_arc) = bridges.get(name) {
      let bridge = bridge_arc.lock().unwrap();
      bridge.is_healthy()
    } else {
      false
    }
  }

  /// 关闭指定的 Bridge
  ///
  /// # Errors
  ///
  /// Bridge 不存在或关闭失败
  pub fn shutdown_bridge(&self, name: &str) -> Result<()> {
    let mut bridges = self.bridges.lock().unwrap();
    if let Some(bridge_arc) = bridges.remove(name) {
      let mut bridge = bridge_arc.lock().unwrap();
      bridge.shutdown()?;
    }
    Ok(())
  }

  /// 关闭所有 Bridge
  pub fn shutdown_all(&self) {
    let mut bridges = self.bridges.lock().unwrap();
    let names: Vec<String> = bridges.keys().cloned().collect();

    for name in names {
      if let Some(bridge_arc) = bridges.remove(&name) {
        let mut bridge = bridge_arc.lock().unwrap();
        if let Err(e) = bridge.shutdown() {
          eprintln!("[BridgeManager] Failed to shutdown bridge '{name}': {e}");
        }
      }
    }
  }

  /// 获取已注册的 Bridge 数量
  #[must_use]
  pub fn bridge_count(&self) -> usize {
    let bridges = self.bridges.lock().unwrap();
    bridges.len()
  }

  /// 获取已注册的工厂数量
  #[must_use]
  pub fn factory_count(&self) -> usize {
    let factories = self.factories.lock().unwrap();
    factories.len()
  }
}

impl Default for BridgeManager {
  fn default() -> Self {
    Self::new()
  }
}

impl Drop for BridgeManager {
  fn drop(&mut self) {
    // 暂时禁用自动清理，调试内存问题
    // let _ = self.shutdown_all();
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use serde_json::json;

  use super::*;
  use crate::bridge::{
    manager::config_bridge::ConfigBridge,
    runtime::{RuntimeType, discovery::RuntimeDiscovery},
  };

  fn get_workspace_root() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR").map_or_else(
      |_| PathBuf::from("."),
      |p| PathBuf::from(p).parent().unwrap().parent().unwrap().to_path_buf(),
    )
  }

  #[test]
  #[ignore = "requires config-bridge process"]
  fn test_bridge_manager_register_and_call() {
    use std::fs;

    use tempfile::tempdir;

    use crate::assets::AssetLocator;

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

    let manager = BridgeManager::new();

    // 注册 config-bridge 工厂
    let config_path_clone = config_path.clone();
    let bridge_script_clone = bridge_script.clone();
    manager.register("config-bridge", move || {
      let discovery = RuntimeDiscovery::new();
      let runtime_info = discovery
        .find_runtime(RuntimeType::NodeJs)?
        .ok_or_else(|| BridgeError::RuntimeNotFound("Node.js".to_string()))?;

      let mut env = HashMap::new();
      env.insert(
        env_keys::CONFIG_PATH.to_string(),
        config_path_clone.to_string_lossy().to_string(),
      );
      env.insert(env_keys::RUNTIME.to_string(), "node".to_string());

      let bridge = ConfigBridge::new(runtime_info, &bridge_script_clone, env)?;
      Ok(Box::new(bridge) as Box<dyn Bridge>)
    });

    // 调用 config.get（懒加载）
    let result = manager
      .call("config-bridge", "config.get", json!({}))
      .expect("Failed to call");

    println!("Result: {result:#}");
    assert!(result.is_object());

    // 显式关闭
    manager.shutdown_bridge("config-bridge").expect("Failed to shutdown");

    println!("✅ BridgeManager test passed");
  }
}
