use std::{path::PathBuf, sync::LazyLock};

use clap::Parser;
#[allow(clippy::wildcard_imports)]
use path_absolutize::*;

/// 全局 CLI 参数实例
pub static GLOBAL_CLI: LazyLock<Cli> = LazyLock::new(|| {
  let cli_raw = Cli::parse();
  cli_raw.validate();
  if let Some(ref paths) = cli_raw.external_plugin_search_path {
    let abs_paths: Vec<PathBuf> = paths.iter().map(|p| p.absolutize().unwrap().to_path_buf()).collect();
    let mut cli = cli_raw.clone();
    cli.external_plugin_search_path = Some(abs_paths);
    return cli;
  }
  cli_raw
});

#[derive(Parser)]
#[command(name = "novelsaga_server")]
#[command(about = "NovelSaga Language Server", long_about = None)]
#[derive(Clone)]
pub struct Cli {
  /// Start as LSP server (communicates via stdin/stdout)
  #[arg(long)]
  pub lsp: bool,
  /// Path to the configuration file
  #[arg(long, short = 'c')]
  pub config: Option<PathBuf>,
  /// Additional plugin search paths
  #[arg(long, short = 'p')]
  pub external_plugin_search_path: Option<Vec<PathBuf>>,
}

impl Cli {
  /// 验证并处理命令行参数
  pub fn validate(&self) {
    if let Some(ref config_path) = self.config
      && !config_path.exists()
    {
      eprintln!("Warning: Config file {} does not exist.", config_path.display());
    }
  }
}
