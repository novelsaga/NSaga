use std::path::PathBuf;

use clap::{Parser, Subcommand};
use path_absolutize::Absolutize;

/// JavaScript 运行时选择
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeChoice {
  /// 自动检测（默认）
  Auto,
  /// 强制使用 Node.js
  Node,
  /// 强制使用 Bun
  Bun,
  /// 强制使用 Deno
  Deno,
}

impl std::str::FromStr for RuntimeChoice {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "auto" => Ok(RuntimeChoice::Auto),
      "node" | "nodejs" => Ok(RuntimeChoice::Node),
      "bun" => Ok(RuntimeChoice::Bun),
      "deno" => Ok(RuntimeChoice::Deno),
      _ => Err(format!("Invalid runtime: '{s}'. Valid options: auto, node, bun, deno",)),
    }
  }
}

/// Subcommands for NovelSaga CLI
#[derive(Subcommand, Clone, Debug)]
pub enum Commands {
  /// Start as LSP server (communicates via stdin/stdout)
  Lsp {},

  /// Initialize a new NovelSaga project
  Init {
    /// Project directory (default: current directory)
    #[arg(default_value = ".")]
    path: PathBuf,
  },

  /// Format NovelSaga configuration files
  Format {
    /// Files to format (default: auto-detect)
    #[arg()]
    files: Vec<PathBuf>,

    /// Check only, don't modify files
    #[arg(long)]
    check: bool,
  },

  /// Check configuration files for errors
  Check {
    /// Files to check (default: auto-detect)
    #[arg()]
    files: Vec<PathBuf>,
  },
}

#[derive(Parser)]
#[command(name = "novelsaga")]
#[command(about = "NovelSaga Language Server", long_about = None)]
#[derive(Clone)]
pub struct Cli {
  /// Choose JavaScript runtime (auto, node, bun, deno)
  #[arg(long, default_value = "auto", value_name = "RUNTIME", global = true)]
  runtime: RuntimeChoice,

  /// Path to the Node.js executable (overrides automatic detection)
  #[arg(long, global = true)]
  node_path: Option<PathBuf>,

  /// Path to the Bun executable (overrides automatic detection)
  #[arg(long, global = true)]
  bun_path: Option<PathBuf>,

  /// Path to the Deno executable (overrides automatic detection)
  #[arg(long, global = true)]
  deno_path: Option<PathBuf>,

  #[command(subcommand)]
  pub command: Option<Commands>,
}

impl Cli {
  pub fn new() -> Self {
    let cli = Cli::parse();
    cli.validate();
    cli
  }

  /// 获取用户选择的运行时
  pub fn get_runtime_choice(&self) -> RuntimeChoice {
    self.runtime
  }

  /// 获取用户指定的 Node.js 可执行文件路径（绝对路径）
  pub fn get_node_path(&self) -> Option<PathBuf> {
    self.node_path.as_ref().map(|p| p.absolutize().unwrap().to_path_buf())
  }

  /// 获取用户指定的 Bun 可执行文件路径（绝对路径）
  pub fn get_bun_path(&self) -> Option<PathBuf> {
    self.bun_path.as_ref().map(|p| p.absolutize().unwrap().to_path_buf())
  }

  /// 获取用户指定的 Deno 可执行文件路径（绝对路径）
  pub fn get_deno_path(&self) -> Option<PathBuf> {
    self.deno_path.as_ref().map(|p| p.absolutize().unwrap().to_path_buf())
  }

  /// 验证并处理命令行参数
  fn validate(&self) {
    // 验证 Node.js 路径
    if let Some(ref node_path) = self.node_path
      && !node_path.try_exists().unwrap_or(false)
    {
      eprintln!("Warning: Node.js executable {} does not exist.", node_path.display());
    }

    // 验证 Bun 路径
    if let Some(ref bun_path) = self.bun_path
      && !bun_path.try_exists().unwrap_or(false)
    {
      eprintln!("Warning: Bun executable {} does not exist.", bun_path.display());
    }

    // 验证 Deno 路径
    if let Some(ref deno_path) = self.deno_path
      && !deno_path.try_exists().unwrap_or(false)
    {
      eprintln!("Warning: Deno executable {} does not exist.", deno_path.display());
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_no_subcommand() {
    let cli = Cli::parse_from(["novelsaga"]);
    assert!(cli.command.is_none(), "Expected no subcommand");
  }

  #[test]
  fn test_parse_lsp_subcommand() {
    let cli = Cli::parse_from(["novelsaga", "lsp"]);
    match &cli.command {
      Some(Commands::Lsp {}) => {}
      _ => panic!("Expected Lsp command, got {:?}", cli.command),
    }
  }

  #[test]
  fn test_parse_init_subcommand() {
    let cli = Cli::parse_from(["novelsaga", "init", "/path/to/project"]);
    match &cli.command {
      Some(Commands::Init { path }) => {
        assert_eq!(path.to_string_lossy(), "/path/to/project");
      }
      _ => panic!("Expected Init command, got {:?}", cli.command),
    }
  }

  #[test]
  fn test_parse_global_runtime_with_subcommand() {
    let cli = Cli::parse_from(["novelsaga", "--runtime", "node", "lsp"]);
    assert_eq!(cli.get_runtime_choice(), RuntimeChoice::Node, "Expected Node runtime");
    match &cli.command {
      Some(Commands::Lsp {}) => {}
      _ => panic!("Expected Lsp command, got {:?}", cli.command),
    }
  }

  #[test]
  fn test_parse_global_runtime_without_subcommand() {
    let cli = Cli::parse_from(["novelsaga", "--runtime", "bun"]);
    assert_eq!(cli.get_runtime_choice(), RuntimeChoice::Bun, "Expected Bun runtime");
    assert!(cli.command.is_none(), "Expected no subcommand");
  }
}
