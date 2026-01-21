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
  },
  /// Build CLI for all platforms
  CliAll,
  /// Build everything (WASM, SO, CLI for all platforms)
  BuildAll,
  /// Generate TypeScript bindings for config structs
  GenTsBindings,
  /// Generate Diplomat bindings (C++ and TypeScript)
  GenDiplomatBindings {
    /// Target language (cpp, ts, or all)
    #[arg(long, default_value = "all")]
    target: String,
  },
  /// Clean all build artifacts
  CleanAll,
  /// Clean WASM artifacts
  CleanWasm,
  /// Clean SO artifacts
  CleanSo,
  /// Clean CLI artifacts
  CleanCli,
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  match cli.command {
    Commands::Wasm => tasks::wasm::build()?,
    Commands::So { target } => tasks::so::build_single(target)?,
    Commands::SoAll => tasks::so::build_all()?,
    Commands::Cli { target } => tasks::cli::build_single(target)?,
    Commands::CliAll => tasks::cli::build_all()?,
    Commands::BuildAll => {
      tasks::wasm::build()?;
      tasks::so::build_all()?;
      tasks::cli::build_all()?;
    }
    Commands::GenTsBindings => tasks::bindings::generate_ts()?,
    Commands::GenDiplomatBindings { target } => match target.as_str() {
      "cpp" => tasks::diplomat::generate_cpp()?,
      "ts" => tasks::diplomat::generate_typescript()?,
      "all" => tasks::diplomat::generate_all()?,
      _ => anyhow::bail!("Unknown diplomat target: {}", target),
    },
    Commands::CleanAll => tasks::clean::clean_all()?,
    Commands::CleanWasm => tasks::clean::clean_wasm()?,
    Commands::CleanSo => tasks::clean::clean_so()?,
    Commands::CleanCli => tasks::clean::clean_cli()?,
  }

  Ok(())
}
