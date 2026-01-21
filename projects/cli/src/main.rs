#![feature(mpmc_channel)]

use novelsaga_core::state::{feat::Feature, init::Initializer};

use crate::args::Cli;
mod args;
mod lsp;
mod plugins;

#[tokio::main(flavor = "current_thread")]
async fn main() {
  // 访问全局 CLI，触发解析和配置加载
  let cli = Cli::new();
  cli.validate();
  let feature = Feature::new(None, None);
  Initializer::init(feature).expect("Failed to initialize");

  if cli.lsp {
    lsp::start().await;
  }
}
