//! Metadata path resolver for locating workspace-specific metadata storage.
//!
//! The resolver handles multiple resolution strategies:
//! 1. **Explicit workspace root** → canonical path: `<workspace>/.cache/novelsaga/sled`
//! 2. **Fallback root from context** → CLI target path, CLI cwd, show target parent, or LSP startup directory

use std::path::{Path, PathBuf};

/// Configuration for metadata path resolution
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ResolutionContext {
  /// Explicit workspace root (highest priority)
  pub workspace_root: Option<PathBuf>,
  /// CLI target path from command arguments
  pub cli_target_path: Option<PathBuf>,
  /// CLI working directory
  pub cli_cwd: Option<PathBuf>,
  /// Show target parent directory
  pub show_target_parent: Option<PathBuf>,
  /// LSP startup directory
  pub lsp_startup_dir: Option<PathBuf>,
}

/// Resolves metadata storage paths
#[allow(dead_code)]
pub struct MetadataResolver;

#[allow(dead_code)]
impl MetadataResolver {
  /// Resolve the canonical metadata path based on context.
  ///
  /// # Resolution Priority
  /// 1. If explicit `workspace_root` is provided → use it
  /// 2. Use first available context path (CLI target, CLI cwd, show parent, LSP startup)
  ///
  /// # Returns
  /// The canonical metadata path: `<workspace>/.cache/novelsaga/sled`
  pub fn resolve(context: &ResolutionContext) -> Result<PathBuf, ResolverError> {
    // Priority 1: Explicit workspace root
    if let Some(ref workspace) = context.workspace_root {
      return Ok(Self::canonical_path(workspace));
    }

    let candidates = [
      context.cli_target_path.as_ref(),
      context.cli_cwd.as_ref(),
      context.show_target_parent.as_ref(),
      context.lsp_startup_dir.as_ref(),
    ];

    for candidate in candidates.iter().flatten() {
      let Some(anchor) = Self::candidate_anchor(candidate) else {
        continue;
      };

      return Ok(Self::canonical_path(&anchor));
    }

    // No valid paths found
    Err(ResolverError::NoValidWorkspaceFound)
  }

  fn candidate_anchor(candidate: &Path) -> Option<PathBuf> {
    if candidate.as_os_str().is_empty() {
      return None;
    }

    if candidate.extension().is_some() {
      return candidate.parent().map(Path::to_path_buf);
    }

    Some(candidate.to_path_buf())
  }

  /// Compute the canonical metadata path for a workspace.
  ///
  /// # Returns
  /// `<workspace>/.cache/novelsaga/sled`
  pub fn canonical_path(workspace: &Path) -> PathBuf {
    workspace.join(".cache/novelsaga/sled")
  }
}

/// Errors that can occur during path resolution
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ResolverError {
  /// No valid workspace root could be determined from any context
  NoValidWorkspaceFound,
  /// Invalid workspace root path
  InvalidWorkspaceRoot(String),
  /// IO error during path resolution
  IoError(String),
}

impl std::fmt::Display for ResolverError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ResolverError::NoValidWorkspaceFound => {
        write!(f, "No valid workspace found in any context")
      }
      ResolverError::InvalidWorkspaceRoot(msg) => {
        write!(f, "Invalid workspace root: {msg}")
      }
      ResolverError::IoError(msg) => {
        write!(f, "IO error during resolution: {msg}")
      }
    }
  }
}

impl std::error::Error for ResolverError {}

#[cfg(test)]
mod tests {
  use tempfile::TempDir;

  use super::*;

  #[test]
  fn test_explicit_workspace_root_returns_canonical_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    let context = ResolutionContext {
      workspace_root: Some(workspace_root.clone()),
      cli_target_path: None,
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(
      resolved, expected,
      "Explicit workspace root should resolve to canonical path"
    );

    Ok(())
  }

  #[test]
  fn test_fallback_to_cli_target_path() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: Some(workspace_root.clone()),
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(resolved, expected, "Should fallback to CLI target path directly");

    Ok(())
  }

  #[test]
  fn test_fallback_to_cli_cwd() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: None,
      cli_cwd: Some(workspace_root.clone()),
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(resolved, expected, "Should fallback to CLI cwd directly");

    Ok(())
  }

  #[test]
  fn test_fallback_to_show_target_parent() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: None,
      cli_cwd: None,
      show_target_parent: Some(workspace_root.clone()),
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(resolved, expected, "Should fallback to show target parent directly");

    Ok(())
  }

  #[test]
  fn test_fallback_to_lsp_startup_dir() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: None,
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: Some(workspace_root.clone()),
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(resolved, expected, "Should fallback to LSP startup dir directly");

    Ok(())
  }

  #[test]
  fn test_cli_target_file_uses_parent_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    let target_file = workspace_root.join("chapter-1.md");
    std::fs::File::create(&target_file)?;

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: Some(target_file),
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(
      resolved, expected,
      "File target should resolve using its parent directory"
    );

    Ok(())
  }

  #[test]
  fn test_canonical_path_construction() {
    let workspace = PathBuf::from("/home/user/my-project");
    let canonical = MetadataResolver::canonical_path(&workspace);
    let expected = PathBuf::from("/home/user/my-project/.cache/novelsaga/sled");

    assert_eq!(
      canonical, expected,
      "Canonical path should be <workspace>/.cache/novelsaga/sled"
    );
  }

  #[test]
  fn test_no_valid_workspace_returns_error() {
    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: None,
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let result = MetadataResolver::resolve(&context);
    assert!(result.is_err(), "Should return error when no valid workspace is found");
    assert!(matches!(result.unwrap_err(), ResolverError::NoValidWorkspaceFound));
  }

  #[test]
  fn test_resolution_priority_explicit_over_fallback() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir1 = TempDir::new()?;
    let temp_dir2 = TempDir::new()?;

    let workspace1 = temp_dir1.path().to_path_buf();
    let workspace2 = temp_dir2.path().to_path_buf();

    let context = ResolutionContext {
      workspace_root: Some(workspace1.clone()),
      cli_target_path: Some(workspace2.clone()),
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace1.join(".cache/novelsaga/sled");

    assert_eq!(
      resolved, expected,
      "Explicit workspace root should take priority over fallback"
    );

    Ok(())
  }

  #[test]
  fn test_resolution_priority_cli_target_over_cwd() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir1 = TempDir::new()?;
    let temp_dir2 = TempDir::new()?;

    let workspace1 = temp_dir1.path().to_path_buf();
    let workspace2 = temp_dir2.path().to_path_buf();

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: Some(workspace1.clone()),
      cli_cwd: Some(workspace2.clone()),
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace1.join(".cache/novelsaga/sled");

    assert_eq!(resolved, expected, "CLI target path should take priority over CLI cwd");

    Ok(())
  }

  #[test]
  fn test_cli_lsp_database_consistency() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace = temp_dir.path().to_path_buf();
    let cli_target_file = workspace.join("file.md");
    std::fs::File::create(&cli_target_file)?;

    let cli_context = ResolutionContext {
      workspace_root: None,
      cli_target_path: Some(cli_target_file),
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let lsp_context = ResolutionContext {
      workspace_root: None,
      cli_target_path: None,
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: Some(workspace.clone()),
    };

    let cli_path = MetadataResolver::resolve(&cli_context)?;
    let lsp_path = MetadataResolver::resolve(&lsp_context)?;

    assert_eq!(
      cli_path, lsp_path,
      "CLI and LSP should resolve to the same canonical database path"
    );
    assert_eq!(
      cli_path,
      workspace.join(".cache/novelsaga/sled"),
      "Canonical database path should be <workspace>/.cache/novelsaga/sled"
    );

    Ok(())
  }
}
