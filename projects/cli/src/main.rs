#![feature(mpmc_channel)]

use std::sync::Arc;

use novelsaga_core::state::{feat::Feature, init::Initializer};

use crate::{args::Cli, bridge::BridgeManager, config::loader::ConfigLoader};

mod args;
mod assets;
mod bridge;
mod config;
mod lsp;
mod plugins;

#[tokio::main(flavor = "current_thread")]
async fn main() {
  // 访问全局 CLI，触发解析和配置加载
  let cli = Cli::new();

  // Initialize Bridge System
  let bridge_manager = Arc::new(BridgeManager::new());
  let config_loader = ConfigLoader::new(bridge_manager.clone(), &cli);

  // 创建 JS/TS Loader 闭包（ConfigLoader 提供）
  let js_loader = Some(config_loader.create_js_loader());
  let ts_loader = Some(config_loader.create_ts_loader());

  // 初始化 Core Feature（提供配置加载能力）
  let feature = Feature::new(js_loader, ts_loader);
  Initializer::init(feature).expect("Failed to initialize");

  if cli.lsp {
    lsp::start().await;
  }
}
