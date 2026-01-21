use std::path::PathBuf;

use clap::Parser;
use path_absolutize::Absolutize;
use version_compare::Version;
use which::which;

const MIN_TS_NODE_VERSION: &str = "23.6";

const MIN_JS_NODE_VERSION: &str = "13.2.0";

#[derive(Parser)]
#[command(name = "novelsaga_server")]
#[command(about = "NovelSaga Language Server", long_about = None)]
#[derive(Clone)]
pub struct Cli {
  /// Start as LSP server (communicates via stdin/stdout)
  #[arg(long)]
  pub lsp: bool,
  /// Path to the Node.js executable
  #[arg(long)]
  node_path: Option<PathBuf>,
}

impl Cli {
  pub fn new() -> Self {
    let cli = Cli::parse();
    cli.validate();
    cli
  }

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

  /// 验证并处理命令行参数
  pub fn validate(&self) {
    // if node_path is provided, judge node.js executable
    if let Some(ref node_path) = self.node_path
      && !node_path.try_exists().unwrap_or(false)
    {
      eprintln!("Warning: Node.js executable {} does not exist.", node_path.display());
    }
  }
}
