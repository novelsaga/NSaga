use std::{
  io::Write,
  path::{Path, PathBuf},
};

use clap::{Parser, Subcommand};
use novelsaga_core::metadata::{
  MetadataEntity,
  parser::{generate_namespace, resolve_type},
};
use serde_json::json;
use walkdir::WalkDir;

use crate::{
  bridge::{rpc::client::RpcClient, transport::unix_socket::UnixSocketTransport},
  metadata::{
    index::IndexManager,
    resolver::{MetadataResolver, ResolutionContext},
  },
};
/// Subcommands for `metadata` operations
#[derive(Subcommand, Clone, Debug)]
pub enum MetadataCommands {
  /// Index a directory for metadata
  Index(IndexCommand),

  /// List all indexed directories
  List(ListCommand),

  /// Show metadata for a specific file
  Show(ShowCommand),
}

/// Index a directory for metadata
#[derive(Parser, Clone, Debug)]
pub struct IndexCommand {
  /// Directory path to index (default: current directory)
  #[arg(default_value = ".")]
  pub path: PathBuf,
}

/// List all indexed directories
#[derive(Parser, Clone, Debug)]
pub struct ListCommand {
  /// Show detailed information
  #[arg(long)]
  pub detailed: bool,
}

/// Show metadata for a specific file
#[derive(Parser, Clone, Debug)]
pub struct ShowCommand {
  /// File path to show metadata for
  #[arg()]
  pub path: PathBuf,
}

/// Handle metadata commands
pub async fn handle_metadata_command(command: MetadataCommands) -> anyhow::Result<()> {
  // Check if LSP is running
  if is_lsp_running() {
    match command {
      MetadataCommands::Index(cmd) => route_to_lsp("novelsaga/index", json!({"path": cmd.path})).await,
      MetadataCommands::List(cmd) => route_to_lsp("novelsaga/list", json!({"detailed": cmd.detailed})).await,
      MetadataCommands::Show(cmd) => route_to_lsp("novelsaga/show", json!({"path": cmd.path})).await,
    }
  } else {
    // LSP not running, execute directly
    match command {
      MetadataCommands::Index(cmd) => handle_index(cmd).await,
      MetadataCommands::List(cmd) => handle_list(cmd).await,
      MetadataCommands::Show(cmd) => handle_show(cmd).await,
    }
  }
}
/// Check if LSP server is running by checking socket file existence
fn is_lsp_running() -> bool {
  let socket_path = std::path::PathBuf::from("/tmp/novelsaga-lsp.sock");
  // Check if socket file exists and try to connect
  if !socket_path.exists() {
    return false;
  }
  // Verify we can actually connect
  UnixSocketTransport::connect(&socket_path).is_ok()
}

/// Route command to LSP server via JSON-RPC
async fn route_to_lsp(command: &str, params: serde_json::Value) -> anyhow::Result<()> {
  let socket_path = std::path::PathBuf::from("/tmp/novelsaga-lsp.sock");
  let command = command.to_string();

  let result = tokio::task::spawn_blocking(move || {
    let transport = UnixSocketTransport::connect(&socket_path)?;
    let mut client = RpcClient::new(Box::new(transport));

    client.call(
      "workspace/executeCommand",
      json!({
        "command": command,
        "arguments": [params]
      }),
    )
  })
  .await??;

  println!("LSP response: {result}");
  Ok(())
}

#[allow(clippy::unused_async)]
async fn handle_index(cmd: IndexCommand) -> anyhow::Result<()> {
  let path = &cmd.path;

  // Validate directory exists
  if !path.exists() {
    anyhow::bail!("Directory does not exist: {}", path.display());
  }
  if !path.is_dir() {
    anyhow::bail!("Path is not a directory: {}", path.display());
  }

  // Determine sled database path using MetadataResolver
  let context = ResolutionContext {
    workspace_root: None,
    cli_target_path: Some(path.clone()),
    cli_cwd: None,
    show_target_parent: None,
    lsp_startup_dir: None,
  };
  let db_path = MetadataResolver::resolve(&context)?;
  println!("📦 Opening index database at: {}", db_path.display());

  // Open IndexManager
  let index_manager = IndexManager::open(&db_path)?;

  // Collect all .md files
  let md_files: Vec<_> = WalkDir::new(path)
    .into_iter()
    .filter_map(std::result::Result::ok)
    .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext.eq_ignore_ascii_case("md")))
    .map(|e| e.path().to_path_buf())
    .collect();

  let total_files = md_files.len();
  println!("🔍 Found {total_files} markdown files to index");

  if total_files == 0 {
    println!("⚠️  No markdown files found in directory");
    return Ok(());
  }

  // Process files with progress tracking
  let mut processed = 0;
  let mut success = 0;
  let mut failed = 0;

  for file_path in md_files {
    processed += 1;
    print!(
      "\r📄 [{}/{}] Processing: {}",
      processed,
      total_files,
      file_path.display()
    );
    std::io::Write::flush(&mut std::io::stdout())?;

    match process_file(&file_path, path, &index_manager) {
      Ok(()) => success += 1,
      Err(e) => {
        eprintln!("\n⚠️  Failed to index {}: {}", file_path.display(), e);
        failed += 1;
      }
    }
  }

  println!(); // New line after progress

  // Flush database
  print!("💾 Flushing database... ");
  std::io::Write::flush(&mut std::io::stdout())?;
  index_manager.flush()?;
  println!("✓");
  // Print summary
  println!("\n📊 Indexing Complete:");
  println!("   Total files: {total_files}");
  println!("   Successful: {success}");
  println!("   Failed: {failed}");
  println!("   Database: {}", db_path.display());

  Ok(())
}

/// Process a single markdown file
/// Process a single markdown file
fn process_file(file_path: &Path, workspace_root: &Path, index_manager: &IndexManager) -> anyhow::Result<()> {
  // Read file content
  let content = std::fs::read_to_string(file_path)?;

  // Parse frontmatter with simple YAML frontmatter delimiter handling
  let (frontmatter, body) = if let Some(rest) = content.strip_prefix("---") {
    if let Some(end) = rest.find("---") {
      let body_str = &rest[end + 3..].trim_start();
      (serde_json::json!({}), body_str.to_string())
    } else {
      (serde_json::json!({}), content)
    }
  } else {
    (serde_json::json!({}), content)
  };

  // Generate ID from file path
  let id = IndexManager::generate_id(&file_path.to_string_lossy());

  // Determine type (frontmatter takes priority, then path inference)
  let type_ = resolve_type(file_path, &frontmatter);

  // Generate namespace from file location
  let namespace = generate_namespace(file_path, workspace_root);

  // Create metadata entity
  let entity = MetadataEntity::new(&id, &type_, &namespace, frontmatter, &body);

  // Index the entity
  index_manager.index_entity(&entity)?;

  Ok(())
}

#[allow(clippy::unused_async)]
async fn handle_list(cmd: ListCommand) -> anyhow::Result<()> {
  // Determine database path using MetadataResolver
  let context = ResolutionContext {
    workspace_root: None,
    cli_target_path: None,
    cli_cwd: Some(std::env::current_dir()?),
    show_target_parent: None,
    lsp_startup_dir: None,
  };
  let cache_dir = MetadataResolver::resolve(&context)?;

  // Open IndexManager
  let index_manager =
    IndexManager::open(&cache_dir).map_err(|e| anyhow::anyhow!("Failed to open index database: {e}"))?;
  // Get all entities using list_all method
  let mut entities: Vec<MetadataEntity> = index_manager
    .list_all()
    .map_err(|e| anyhow::anyhow!("Failed to list entities: {e}"))?;

  if entities.is_empty() {
    println!("No indexed entities found.");
    println!("Use 'novelsaga index <path>' to index a directory.");
    return Ok(());
  }

  // Sort entities by type, then namespace, then id
  entities.sort_by(|a, b| (&a.type_, &a.namespace, &a.id).cmp(&(&b.type_, &b.namespace, &b.id)));

  if cmd.detailed {
    // Detailed table output with frontmatter summary
    print_detailed_table(&entities)?;
  } else {
    // Simple table output
    print_simple_table(&entities)?;
  }

  println!("\nTotal: {} entities", entities.len());

  Ok(())
}

/// Prints a simple table with ID, Type, and Namespace columns
fn print_simple_table(entities: &[MetadataEntity]) -> anyhow::Result<()> {
  // Calculate column widths
  let id_width = entities.iter().map(|e| e.id.len()).max().unwrap_or(10).max(10);
  let type_width = entities.iter().map(|e| e.type_.len()).max().unwrap_or(10).max(10);
  let ns_width = entities.iter().map(|e| e.namespace.len()).max().unwrap_or(10).max(10);

  let total_width = id_width + type_width + ns_width + 10; // +10 for borders and spacing

  // Print header separator
  println!("{}", "─".repeat(total_width));

  // Print header
  println!(
    "│ {:id_width$} │ {:type_width$} │ {:ns_width$} │",
    "ID", "TYPE", "NAMESPACE"
  );

  // Print header separator
  println!("├{0:─<id_width$}─┼─{0:─<type_width$}─┼─{0:─<ns_width$}─┤", "─");

  // Print rows
  let mut stdout = std::io::stdout().lock();
  for entity in entities {
    writeln!(
      stdout,
      "│ {:id_width$} │ {:type_width$} │ {:ns_width$} │",
      truncate(&entity.id, id_width),
      truncate(&entity.type_, type_width),
      truncate(&entity.namespace, ns_width),
    )?;
  }

  // Print footer separator
  println!("└{0:─<id_width$}─┴─{0:─<type_width$}─┴─{0:─<ns_width$}─┘", "─");

  Ok(())
}

/// Prints a detailed table including frontmatter summary
fn print_detailed_table(entities: &[MetadataEntity]) -> anyhow::Result<()> {
  // Calculate column widths
  let id_width = entities.iter().map(|e| e.id.len()).max().unwrap_or(10).clamp(10, 20);
  let type_width = entities.iter().map(|e| e.type_.len()).max().unwrap_or(10).clamp(10, 15);
  let ns_width = entities
    .iter()
    .map(|e| e.namespace.len())
    .max()
    .unwrap_or(10)
    .clamp(10, 20);
  let summary_width = 40;

  let total_width = id_width + type_width + ns_width + summary_width + 16; // +16 for borders and spacing

  // Print header separator
  println!("{}", "─".repeat(total_width));

  // Print header
  println!(
    "│ {:id_width$} │ {:type_width$} │ {:ns_width$} │ {:summary_width$} │",
    "ID", "TYPE", "NAMESPACE", "FRONTMATTER SUMMARY"
  );

  // Print header separator
  println!(
    "├{0:─<id_width$}─┼─{0:─<type_width$}─┼─{0:─<ns_width$}─┼─{0:─<summary_width$}─┤",
    "─"
  );

  // Print rows
  let mut stdout = std::io::stdout().lock();
  for entity in entities {
    let summary = format_frontmatter_summary(&entity.frontmatter, summary_width);
    writeln!(
      stdout,
      "│ {:id_width$} │ {:type_width$} │ {:ns_width$} │ {:summary_width$} │",
      truncate(&entity.id, id_width),
      truncate(&entity.type_, type_width),
      truncate(&entity.namespace, ns_width),
      summary,
    )?;
  }

  // Print footer separator
  println!(
    "└{0:─<id_width$}─┴─{0:─<type_width$}─┴─{0:─<ns_width$}─┴─{0:─<summary_width$}─┘",
    "─"
  );

  Ok(())
}

/// Formats frontmatter as a summary string
fn format_frontmatter_summary(frontmatter: &serde_json::Value, max_len: usize) -> String {
  match frontmatter {
    serde_json::Value::Object(map) if !map.is_empty() => {
      let items: Vec<String> = map
        .iter()
        .take(3) // Show at most 3 fields
        .map(|(k, v)| {
          let v_str = match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
            _ => "...".to_string(),
          };
          let v_truncated = if v_str.len() > 15 {
            format!("{}...", &v_str[..12])
          } else {
            v_str
          };
          format!("{k}: {v_truncated}")
        })
        .collect();

      let result = items.join(", ");
      if map.len() > 3 {
        format!("{result}, ...")
      } else {
        result
      }
    }
    _ => "(empty)".to_string(),
  }
  .chars()
  .take(max_len)
  .collect()
}

/// Truncates a string to fit within the given width
fn truncate(s: &str, max_width: usize) -> String {
  if s.len() <= max_width {
    s.to_string()
  } else {
    format!("{}...", &s[..max_width.saturating_sub(3)])
  }
}

#[allow(clippy::unused_async)]
async fn handle_show(cmd: ShowCommand) -> anyhow::Result<()> {
  use anyhow::Context;

  // Get canonical path for consistent ID generation
  let canonical_path = cmd
    .path
    .canonicalize()
    .with_context(|| format!("File not found: {}", cmd.path.display()))?;
  let path_str = canonical_path.to_string_lossy();

  // Generate entity ID from path using blake3 hash
  let entity_id = IndexManager::generate_id(&path_str);

  // Determine index database path using MetadataResolver
  let show_target_parent = canonical_path.parent().map(std::path::Path::to_path_buf);
  let context = ResolutionContext {
    workspace_root: None,
    cli_target_path: None,
    cli_cwd: None,
    show_target_parent,
    lsp_startup_dir: None,
  };
  let cache_dir = MetadataResolver::resolve(&context)?;

  // Open index manager
  let index_manager =
    IndexManager::open(&cache_dir).map_err(|e| anyhow::anyhow!("Failed to open index database: {e}"))?;

  // Query entity by ID
  if let Some(entity) = index_manager.get_by_id(&entity_id)? {
    // Display entity as pretty JSON
    let json_output = serde_json::to_string_pretty(&entity).context("Failed to serialize metadata entity")?;
    println!("{json_output}");
  } else {
    eprintln!("No metadata found for: {}", cmd.path.display());
    eprintln!("  Entity ID: {entity_id}");
    eprintln!("  Run 'novelsaga index' to index this directory first.");
    std::process::exit(1);
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_index_command() {
    let cmd = IndexCommand {
      path: PathBuf::from("/path/to/dir"),
    };
    assert_eq!(cmd.path.to_string_lossy(), "/path/to/dir");
  }

  #[test]
  fn test_list_command() {
    let cmd = ListCommand { detailed: false };
    assert!(!cmd.detailed);
  }

  #[test]
  fn test_list_command_detailed() {
    let cmd = ListCommand { detailed: true };
    assert!(cmd.detailed);
  }

  #[test]
  fn test_show_command() {
    let cmd = ShowCommand {
      path: PathBuf::from("/path/to/file"),
    };
    assert_eq!(cmd.path.to_string_lossy(), "/path/to/file");
  }
}
