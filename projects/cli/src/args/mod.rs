use std::{path::PathBuf, sync::LazyLock};

use clap::Parser;
#[allow(clippy::wildcard_imports)]
use path_absolutize::*;
use version_compare::Version;
use which::which;

const MIN_TS_NODE_VERSION: &str = "23.6";

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
  /// Path to the Node.js executable
  #[arg(long)]
  node_path: Option<PathBuf>,
}

impl Cli {
  /// 获取 node.js 可执行文件路径
  /// 如果参数没有提供，则从主机环境中查找node命令
  /// 最后返回None
  pub fn get_node_path(&self) -> Option<PathBuf> {
    if let Some(ref node_path) = self.node_path {
      // path 转为绝对路径
      return Some(node_path.absolutize().unwrap().to_path_buf());
    }
    if let Ok(path) = which("node") {
      return Some(path.absolutize().unwrap().to_path_buf());
    }
    None
  }

  /// 获取node版本号 并去掉前缀v
  pub fn get_node_version(&self) -> Option<String> {
    let node_path = self.get_node_path()?;
    let output = std::process::Command::new(node_path).arg("-v").output().ok()?;
    if output.status.success() {
      let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
      Some(version.trim_start_matches('v').to_string())
    } else {
      None
    }
  }

  /// 判断js支持
  pub fn is_js_supported(&self) -> bool {
    self.get_node_version().is_some()
  }

  /// 判断版本号是否大于等于23.6(原生支持ts)
  pub fn is_ts_supported(&self) -> bool {
    let min_ts_node_version = Version::from(MIN_TS_NODE_VERSION).unwrap();
    if let Some(version_str) = self.get_node_version() {
      let version_str = version_str.trim_start_matches('v');
      if let Some(version) = Version::from(version_str) {
        return version >= min_ts_node_version;
      }
    }
    false
  }
}

impl Cli {
  /// 验证并处理命令行参数
  pub fn validate(&self) {
    if let Some(ref config_path) = self.config
      && !config_path.try_exists().unwrap_or(false)
    {
      eprintln!("Warning: Config file {} does not exist.", config_path.display());
    }
    // if node_path is provided, judge node.js executable
    if let Some(ref node_path) = self.node_path
      && !node_path.try_exists().unwrap_or(false)
    {
      eprintln!("Warning: Node.js executable {} does not exist.", node_path.display());
    }
  }
}
