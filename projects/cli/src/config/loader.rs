use std::{
  collections::HashMap,
  error::Error,
  path::{Path, PathBuf},
  sync::Arc,
};

use anyhow::Result;
use serde_json::json;

use crate::{
  args::{Cli, RuntimeChoice},
  bridge::{BridgeManager, ConfigBridge, RuntimeDiscovery, RuntimeType, env_keys},
};

/// Type alias for the config loader closure returned to Core
pub type ConfigLoaderFn =
  Arc<dyn Fn(&str) -> Result<HashMap<String, serde_json::Value>, Box<dyn Error + Send + Sync>> + Send + Sync>;

/// Internal context for loading script configs (reduces function parameter count)
struct LoaderContext<'a> {
  manager: &'a Arc<BridgeManager>,
  runtime_discovery: RuntimeDiscovery,
  runtime_choice: RuntimeChoice,
  node_path: Option<&'a PathBuf>,
  bun_path: Option<&'a PathBuf>,
  deno_path: Option<&'a PathBuf>,
}

/// CLI 配置加载器：仅负责通过 Bridge 加载 JS/TS 配置
///
/// 架构说明：
/// - Core 的 `ConfigManager` 负责查找配置文件和加载静态配置
/// - CLI 的 `ConfigLoader` 仅提供 `js_loader/ts_loader` 闭包给 Core
/// - 不要在 CLI 中重复实现配置查找逻辑
#[derive(Clone)]
pub struct ConfigLoader {
  manager: Arc<BridgeManager>,
  runtime_discovery: RuntimeDiscovery,
  runtime_choice: RuntimeChoice,
  node_path: Option<PathBuf>,
  bun_path: Option<PathBuf>,
  deno_path: Option<PathBuf>,
}

impl ConfigLoader {
  pub fn new(manager: Arc<BridgeManager>, cli: &Cli) -> Self {
    Self {
      manager,
      runtime_discovery: RuntimeDiscovery::new(),
      runtime_choice: cli.get_runtime_choice(),
      node_path: cli.get_node_path(),
      bun_path: cli.get_bun_path(),
      deno_path: cli.get_deno_path(),
    }
  }

  /// 创建 JS 配置加载闭包，供 Core 的 Feature 使用
  pub fn create_js_loader(&self) -> ConfigLoaderFn {
    let manager = self.manager.clone();
    let runtime_discovery = self.runtime_discovery;
    let runtime_choice = self.runtime_choice;
    let node_path = self.node_path.clone();
    let bun_path = self.bun_path.clone();
    let deno_path = self.deno_path.clone();

    Arc::new(move |config_path: &str| {
      let ctx = LoaderContext {
        manager: &manager,
        runtime_discovery,
        runtime_choice,
        node_path: node_path.as_ref(),
        bun_path: bun_path.as_ref(),
        deno_path: deno_path.as_ref(),
      };
      Self::load_script_config_impl(&ctx, config_path, false).map_err(std::convert::Into::into)
    })
  }

  /// 创建 TS 配置加载闭包，供 Core 的 Feature 使用
  pub fn create_ts_loader(&self) -> ConfigLoaderFn {
    let manager = self.manager.clone();
    let runtime_discovery = self.runtime_discovery;
    let runtime_choice = self.runtime_choice;
    let node_path = self.node_path.clone();
    let bun_path = self.bun_path.clone();
    let deno_path = self.deno_path.clone();

    Arc::new(move |config_path: &str| {
      let ctx = LoaderContext {
        manager: &manager,
        runtime_discovery,
        runtime_choice,
        node_path: node_path.as_ref(),
        bun_path: bun_path.as_ref(),
        deno_path: deno_path.as_ref(),
      };
      Self::load_script_config_impl(&ctx, config_path, true).map_err(std::convert::Into::into)
    })
  }

  /// 通用脚本配置加载实现
  fn load_script_config_impl(
    ctx: &LoaderContext<'_>,
    config_path: &str,
    is_typescript: bool,
  ) -> Result<HashMap<String, serde_json::Value>> {
    let bridge_script = Self::get_bridge_script_path()?;

    // 根据用户选择确定运行时类型和路径
    let (preferred_runtime, user_path) = match ctx.runtime_choice {
      RuntimeChoice::Auto => (None, None),
      RuntimeChoice::Node => (Some(RuntimeType::NodeJs), ctx.node_path.cloned()),
      RuntimeChoice::Bun => (Some(RuntimeType::Bun), ctx.bun_path.cloned()),
      RuntimeChoice::Deno => (Some(RuntimeType::Deno), ctx.deno_path.cloned()),
    };

    let runtime_info = ctx
      .runtime_discovery
      .find_runtime_with_preference(preferred_runtime, user_path)
      .map_err(|e| anyhow::anyhow!("Failed to find suitable runtime: {e}"))?;

    // Convert to absolute path
    let config_path_buf = Path::new(config_path);
    let config_path_abs = if config_path_buf.is_absolute() {
      config_path_buf.to_path_buf()
    } else {
      config_path_buf.canonicalize()?
    };

    let project_root = config_path_abs.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();

    let ext = config_path_abs.extension().and_then(|e| e.to_str()).unwrap_or("");

    let is_cjs = ext == "cjs" || ext == "cts";

    let mut env = HashMap::new();
    env.insert(
      env_keys::CONFIG_PATH.to_string(),
      config_path_abs.to_string_lossy().to_string(),
    );
    env.insert(
      env_keys::PROJECT_ROOT.to_string(),
      project_root.to_string_lossy().to_string(),
    );
    env.insert(env_keys::CONFIG_IS_TYPESCRIPT.to_string(), is_typescript.to_string());
    env.insert(env_keys::CONFIG_IS_COMMONJS.to_string(), is_cjs.to_string());
    env.insert(env_keys::IS_DEV.to_string(), "true".to_string());

    // Prepare factory closure
    let runtime_info = runtime_info.clone();
    let bridge_script = bridge_script.clone();
    let env_clone = env.clone();

    // Register the "config" bridge with unique name per config path
    let bridge_name = format!("config_{}", config_path_abs.to_string_lossy().replace(['/', '\\'], "_"));

    ctx.manager.register(&bridge_name, move || {
      let bridge = ConfigBridge::new(runtime_info.clone(), &bridge_script, env_clone.clone())?;
      Ok(Box::new(bridge))
    });

    // Call config.get
    let result_value = ctx
      .manager
      .call(&bridge_name, "config.get", json!({}))
      .map_err(|e| anyhow::anyhow!("Bridge call failed: {e}"))?;

    // Return as HashMap for Core's LoaderFn signature
    if let serde_json::Value::Object(map) = result_value {
      Ok(map.into_iter().collect())
    } else {
      anyhow::bail!("Config bridge returned non-object value")
    }
  }

  fn get_bridge_script_path() -> Result<PathBuf> {
    use crate::assets::AssetLocator;

    AssetLocator::locate("js/dist/config-bridge.js").map_err(|e| anyhow::anyhow!("{e}"))
  }
}

#[cfg(test)]
mod tests {
  use std::fs;

  use clap::Parser;
  use tempfile::tempdir;

  use super::*;

  #[test]
  fn test_js_loader_closure() -> Result<()> {
    // 1. Setup temp dir
    let dir = tempdir()?;
    let config_path = dir.path().join("novelsaga.config.mjs");

    // 2. Create a simple JS config (ESM)
    let content = r#"
      export default {
        workspace: {
          cache_dir: ".novelsaga/cache",
          novelsaga_dir: ".novelsaga"
        },
        fmt: {
          indent_spaces: 2,
          blank_lines_between_paragraphs: 2
        }
      };
    "#;
    fs::write(&config_path, content)?;

    // 3. Create mock CLI with default settings
    let cli = crate::args::Cli::parse_from(["novelsaga"]);

    // 4. Create loader and get js_loader closure
    let manager = Arc::new(BridgeManager::new());
    let loader = ConfigLoader::new(manager, &cli);
    let js_loader = loader.create_js_loader();

    // 4. Call the closure (as Core would)
    let result = js_loader(config_path.to_str().unwrap()).map_err(|e| anyhow::anyhow!("Loader failed: {e}"))?;

    // 5. Verify structure
    assert!(result.contains_key("workspace"));
    assert!(result.contains_key("fmt"));

    Ok(())
  }
}
