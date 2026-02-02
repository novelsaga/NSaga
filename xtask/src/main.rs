mod tasks;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "NovelSaga build automation tasks", long_about = None)]
struct Cli {
  #[command(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  /// Build JavaScript bridges (config-bridge, etc.)
  BuildJs {
    /// Force rebuild even if files are up-to-date
    #[arg(long)]
    force: bool,

    /// Only copy built assets to out/cli, skip building
    #[arg(long)]
    copy_only: bool,
  },
  /// Build WASM package from core
  Wasm,
  /// Build Shared Library (.so/.dll/.dylib) for a single target
  So {
    /// Target platform (e.g., linux-x64, windows-arm64)
    #[arg(long)]
    target: Option<String>,
  },
  /// Build Shared Library for all platforms
  SoAll,
  /// Build CLI for a single target platform
  Cli {
    /// Target platform (e.g., linux-x64, windows-arm64)
    #[arg(long)]
    target: Option<String>,

    /// Asset install directory (embedded at compile time for release builds)
    #[arg(long, default_value = "/usr/share/novelsaga/assets")]
    asset_install_dir: String,

    /// Build in release mode
    #[arg(long)]
    release: bool,

    /// Skip building JavaScript bridges (useful when JS is pre-built)
    #[arg(long)]
    skip_js: bool,
  },
  /// Build CLI for all platforms
  CliAll {
    /// Asset install directory (embedded at compile time for release builds)
    #[arg(long, default_value = "/usr/share/novelsaga/assets")]
    asset_install_dir: String,

    /// Build in release mode
    #[arg(long)]
    release: bool,

    /// Skip building JavaScript bridges (useful when JS is pre-built)
    #[arg(long)]
    skip_js: bool,
  },
  /// Build everything (WASM, SO, CLI for all platforms)
  BuildAll {
    /// Asset install directory (embedded at compile time for release builds)
    #[arg(long, default_value = "/usr/share/novelsaga/assets")]
    asset_install_dir: String,

    /// Build in release mode
    #[arg(long)]
    release: bool,

    /// Skip building JavaScript bridges (useful when JS is pre-built)
    #[arg(long)]
    skip_js: bool,
  },
  /// Generate TypeScript bindings for config structs
  GenTsBindings,
  /// Generate Diplomat bindings (C++ and TypeScript)
  GenDiplomatBindings {
    /// Target language (cpp, ts, or all)
    #[arg(long, default_value = "all")]
    target: String,
  },
  /// Run end-to-end tests for CLI
  E2e,
  /// Clean all build artifacts
  CleanAll,
  /// Clean WASM artifacts
  CleanWasm,
  /// Clean SO artifacts
  CleanSo,
  /// Clean CLI artifacts
  CleanCli,
  /// Clean JavaScript bridge artifacts
  CleanJs,
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Commands::BuildJs { force, copy_only } => {
      if copy_only {
        tasks::build::copy_only()?;
      } else if force {
        tasks::build::build_all_force()?;
      } else {
        tasks::build::build_all()?;
      }
    }
    Commands::Wasm => tasks::wasm::build()?,
    Commands::So { target } => tasks::so::build_single(target)?,
    Commands::SoAll => tasks::so::build_all()?,
    Commands::Cli {
      target,
      asset_install_dir,
      release,
      skip_js,
    } => tasks::cli::build_single(target, &asset_install_dir, release, skip_js)?,
    Commands::CliAll {
      asset_install_dir,
      release,
      skip_js,
    } => tasks::cli::build_all(&asset_install_dir, release, skip_js)?,
    Commands::BuildAll {
      asset_install_dir,
      release,
      skip_js,
    } => {
      if !skip_js {
        tasks::build::build_all()?;
      }
      tasks::wasm::build()?;
      tasks::so::build_all()?;
      tasks::cli::build_all(&asset_install_dir, release, true)?; // 避免重复构建 JS
    }
    Commands::GenTsBindings => tasks::bindings::generate_ts()?,
    Commands::GenDiplomatBindings { target } => match target.as_str() {
      "cpp" => tasks::diplomat::generate_cpp()?,
      "ts" => tasks::diplomat::generate_typescript()?,
      "all" => tasks::diplomat::generate_all()?,
      _ => anyhow::bail!("Unknown diplomat target: {}", target),
    },
    Commands::E2e => tasks::test::cli::run_e2e_tests()?,
    Commands::CleanAll => tasks::clean::clean_all()?,
    Commands::CleanWasm => tasks::clean::clean_wasm()?,
    Commands::CleanSo => tasks::clean::clean_so()?,
    Commands::CleanCli => tasks::clean::clean_cli()?,
    Commands::CleanJs => tasks::build::clean()?,
  }

  Ok(())
}
