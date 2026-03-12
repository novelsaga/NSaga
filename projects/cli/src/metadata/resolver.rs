//! Metadata path resolver for locating workspace-specific metadata storage.
//!
//! The resolver handles multiple resolution strategies:
//! 1. **Explicit workspace root** → canonical path: `<workspace>/.cache/novelsaga/sled`
//! 2. **Fallback root from context** → CLI target path, CLI cwd, show target parent, or LSP startup directory
//! 3. **metadata/ heuristic** → recognizes `metadata/` directory as valid workspace marker
//! 4. **Legacy migration** → detects `.novelsaga/cache/index` and migrates to canonical path

use std::{
  fs,
  path::{Path, PathBuf},
};

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
  /// 2. If `metadata/` directory is found in any context path → use its parent
  /// 3. If `.novelsaga/cache/index` is found → migrate to canonical path
  /// 4. Use first available context path (CLI target, CLI cwd, show parent, LSP startup)
  ///
  /// # Returns
  /// The canonical metadata path: `<workspace>/.cache/novelsaga/sled`
  pub fn resolve(context: &ResolutionContext) -> Result<PathBuf, ResolverError> {
    // Priority 1: Explicit workspace root
    if let Some(ref workspace) = context.workspace_root {
      return Ok(Self::canonical_path(workspace));
    }

    // Priority 2-4: Check all context paths for legacy or metadata/ marker
    let candidates = [
      context.cli_target_path.as_ref(),
      context.cli_cwd.as_ref(),
      context.show_target_parent.as_ref(),
      context.lsp_startup_dir.as_ref(),
    ];

    for candidate in candidates.iter().flatten() {
      // Walk upward from candidate, checking legacy before metadata at EACH level
      let mut current = if candidate.is_dir() {
        (*candidate).clone()
      } else {
        match candidate.parent() {
          Some(p) => p.to_path_buf(),
          None => continue,
        }
      };

      loop {
        // Check for legacy .novelsaga/cache/index at THIS level
        let legacy_check = current.join(".novelsaga/cache/index");
        if legacy_check.exists() {
          if let Err(e) = Self::migrate_legacy_data(&current) {
            eprintln!("[novelsaga] Warning: legacy migration failed: {e}");
          }
          return Ok(Self::canonical_path(&current));
        }

        // Check for metadata/ marker at THIS level
        let metadata_dir = current.join("metadata");
        if metadata_dir.is_dir() {
          return Ok(Self::canonical_path(&current));
        }

        // Move up to parent
        match current.parent() {
          Some(parent) => current = parent.to_path_buf(),
          None => break,
        }
      }
    }

    // No valid paths found
    Err(ResolverError::NoValidWorkspaceFound)
  }

  /// Compute the canonical metadata path for a workspace.
  ///
  /// # Returns
  /// `<workspace>/.cache/novelsaga/sled`
  pub fn canonical_path(workspace: &Path) -> PathBuf {
    workspace.join(".cache/novelsaga/sled")
  }

  /// Migrate legacy `.novelsaga/cache/index` data to canonical path.
  ///
  /// Migration only occurs when:
  /// - Legacy path `.novelsaga/cache/index` exists
  /// - Canonical path `.cache/novelsaga/sled` does NOT already exist
  ///
  /// # Errors
  /// Returns `ResolverError::IoError` if filesystem operations fail.
  pub fn migrate_legacy_data(workspace: &Path) -> Result<(), ResolverError> {
    let legacy_path = workspace.join(".novelsaga/cache/index");
    let canonical = Self::canonical_path(workspace);

    // Only migrate when canonical doesn't exist but legacy does
    if canonical.exists() {
      eprintln!(
        "[novelsaga] Canonical path already exists, skipping migration: {}",
        canonical.display()
      );
      return Ok(());
    }

    if !legacy_path.exists() {
      return Ok(());
    }

    eprintln!(
      "[novelsaga] Migrating legacy metadata from {} to {}",
      legacy_path.display(),
      canonical.display()
    );

    // Create parent directories for canonical path
    if let Some(parent) = canonical.parent() {
      fs::create_dir_all(parent)
        .map_err(|e| ResolverError::IoError(format!("Failed to create canonical directory: {e}")))?;
    }

    // Move (rename) legacy data to canonical location
    fs::rename(&legacy_path, &canonical).map_err(|e| {
      ResolverError::IoError(format!(
        "Failed to migrate legacy data from {} to {}: {}",
        legacy_path.display(),
        canonical.display(),
        e
      ))
    })?;

    eprintln!("[novelsaga] Migration complete");
    Ok(())
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
  use std::fs;

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

    // Create metadata/ marker to simulate valid workspace
    fs::create_dir(workspace_root.join("metadata"))?;

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: Some(workspace_root.clone()),
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(
      resolved, expected,
      "Should fallback to CLI target path with metadata/ marker"
    );

    Ok(())
  }

  #[test]
  fn test_fallback_to_cli_cwd() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Create metadata/ marker
    fs::create_dir(workspace_root.join("metadata"))?;

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: None,
      cli_cwd: Some(workspace_root.clone()),
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(resolved, expected, "Should fallback to CLI cwd with metadata/ marker");

    Ok(())
  }

  #[test]
  fn test_fallback_to_show_target_parent() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Create metadata/ marker
    fs::create_dir(workspace_root.join("metadata"))?;

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: None,
      cli_cwd: None,
      show_target_parent: Some(workspace_root.clone()),
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(
      resolved, expected,
      "Should fallback to show target parent with metadata/ marker"
    );

    Ok(())
  }

  #[test]
  fn test_fallback_to_lsp_startup_dir() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Create metadata/ marker
    fs::create_dir(workspace_root.join("metadata"))?;

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: None,
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: Some(workspace_root.clone()),
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(
      resolved, expected,
      "Should fallback to LSP startup dir with metadata/ marker"
    );

    Ok(())
  }

  #[test]
  fn test_metadata_heuristic_finds_metadata_directory() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Create metadata/ subdirectory
    fs::create_dir(workspace_root.join("metadata"))?;

    // Create a nested file path to search from
    let nested_file = workspace_root.join("metadata").join("file.md");
    fs::create_dir_all(nested_file.parent().unwrap())?;
    fs::File::create(&nested_file)?;

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: Some(nested_file),
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(resolved, expected, "Should recognize metadata/ as workspace marker");

    Ok(())
  }

  #[test]
  fn test_legacy_migration_detects_novelsaga_cache_index() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let workspace_root = temp_dir.path().to_path_buf();

    // Create legacy .novelsaga/cache/index path
    fs::create_dir_all(workspace_root.join(".novelsaga/cache"))?;
    fs::File::create(workspace_root.join(".novelsaga/cache/index"))?;

    let context = ResolutionContext {
      workspace_root: None,
      cli_target_path: Some(workspace_root.clone()),
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: None,
    };

    let resolved = MetadataResolver::resolve(&context)?;
    let expected = workspace_root.join(".cache/novelsaga/sled");

    assert_eq!(
      resolved, expected,
      "Should detect and migrate legacy .novelsaga/cache/index"
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

    // Create metadata/ marker in workspace2
    fs::create_dir(workspace2.join("metadata"))?;

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

    // Create metadata/ marker in both
    fs::create_dir(workspace1.join("metadata"))?;
    fs::create_dir(workspace2.join("metadata"))?;

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
}
