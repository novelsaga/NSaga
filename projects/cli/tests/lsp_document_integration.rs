use std::process::Stdio;

use tempfile::TempDir;
use tokio::process::Command;

/// Test context for LSP server integration tests
/// Spawns the novelsaga LSP server and provides infrastructure for client tests
pub struct LspTestContext {
  workspace_root: TempDir,
  _child: tokio::process::Child,
}

impl LspTestContext {
  /// Create a new LSP test context
  ///
  /// This spawns the novelsaga LSP server in a separate process.
  /// The LSP server communicates via stdin/stdout using JSON-RPC 2.0.
  ///
  /// # Errors
  /// Returns an error if the LSP server process cannot be spawned.
  pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
    // Create temporary workspace directory
    let workspace_root = TempDir::new()?;
    let _workspace_path = workspace_root.path().to_path_buf();

    // Spawn the LSP server process
    let child = Command::new("cargo")
      .args(["run", "--bin", "novelsaga", "--", "lsp"])
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::piped())
      .kill_on_drop(true)
      .spawn()?;

    Ok(LspTestContext {
      workspace_root,
      _child: child,
    })
  }

  /// Get the workspace root path
  #[must_use]
  pub fn workspace_root(&self) -> &std::path::Path {
    self.workspace_root.path()
  }
}

impl Drop for LspTestContext {
  fn drop(&mut self) {
    // Process is killed via kill_on_drop(true) in Command
    // This cleanup ensures any remaining resources are freed
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  #[ignore = "requires novelsaga binary to be built"]
  async fn test_lsp_context_creation() {
    let context = LspTestContext::new();
    assert!(context.is_ok(), "Failed to create LSP context");

    let context = context.unwrap();
    assert!(context.workspace_root().exists());
  }
}
