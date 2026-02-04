#![feature(mpmc_channel)]

use std::sync::Arc;

use novelsaga_core::state::{feat::Feature, init::Initializer};

use crate::{
  args::{Cli, Commands},
  bridge::BridgeManager,
  config::loader::ConfigLoader,
};

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

  match &cli.command {
    Some(Commands::Lsp {}) => {
      lsp::start().await;
    }
    Some(Commands::Init { path }) => {
      todo!("Init command not implemented yet. Path: {:?}", path);
    }
    Some(Commands::Format { files, check }) => {
      todo!(
        "Format command not implemented yet. Files: {:?}, Check: {}",
        files,
        check
      );
    }
    Some(Commands::Check { files }) => {
      todo!("Check command not implemented yet. Files: {:?}", files);
    }
    None => {
      print_status_info(&cli);
    }
  }
}

fn print_status_info(cli: &Cli) {
  println!("NovelSaga v{}", env!("CARGO_PKG_VERSION"));
  println!();
  println!("Runtime Configuration:");
  println!("  Selected: {:?}", cli.get_runtime_choice());
  if let Some(path) = cli.get_node_path() {
    println!("  Node.js: {}", path.display());
  }
  if let Some(path) = cli.get_bun_path() {
    println!("  Bun: {}", path.display());
  }
  if let Some(path) = cli.get_deno_path() {
    println!("  Deno: {}", path.display());
  }
  println!();
  println!("Config: not loaded (run in project directory)");
  println!();
  println!("Run 'novelsaga --help' for available commands.");
}
