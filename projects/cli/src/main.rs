mod args;
mod config;
mod core;
mod home_path;
mod lsp;
mod plugins;

use args::GLOBAL_CLI;

#[tokio::main(flavor = "current_thread")]
async fn main() {
  // 访问全局 CLI，触发解析和配置加载
  let cli = &*GLOBAL_CLI;

  if cli.lsp {
    lsp::start().await;
  }
}
