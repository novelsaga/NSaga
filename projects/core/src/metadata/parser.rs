use std::path::Path;

use serde_json::Value;

/// Infers the type from a file path based on its directory structure.
///
/// # Rules
/// - `metadata/*.md` → "metadata"
/// - `metadata/characters/*.md` → "character"
/// - `metadata/scenes/*.md` → "scene"
/// - Others: singularize parent folder name
///
/// # Arguments
/// * `path` - File path to infer type from
///
/// # Returns
/// Inferred type as a String
///
/// # Examples
/// ```ignore
/// let path = Path::new("metadata/characters/hero.md");
/// assert_eq!(infer_type_from_path(path), "character");
/// ```
#[must_use]
pub fn infer_type_from_path(path: &Path) -> String {
  let components: Vec<&std::ffi::OsStr> = path
    .components()
    .filter_map(|c| {
      if let std::path::Component::Normal(n) = c {
        Some(n)
      } else {
        None
      }
    })
    .collect();

  // If no components, default to "metadata"
  if components.is_empty() {
    return "metadata".to_string();
  }

  // Get parent directory name (second to last component, before filename)
  if components.len() >= 2 {
    let parent_name = components[components.len() - 2].to_string_lossy().to_string();

    // Handle special cases for nested metadata directories
    if parent_name == "characters" {
      return "character".to_string();
    }
    if parent_name == "scenes" {
      return "scene".to_string();
    }

    // Singularize the parent directory name
    return singularize(&parent_name);
  }

  // Default: singularize filename without extension
  let filename = components
    .last()
    .map(|c| c.to_string_lossy())
    .unwrap_or_default()
    .to_string();

  let name_without_ext = filename.split('.').next().unwrap_or(&filename).to_string();

  singularize(&name_without_ext)
}

/// Converts a plural word to singular form using simple rules.
///
/// # Simple Rules
/// - "ies" → "y" (e.g., "characters" → "character")
/// - "s" → "" (e.g., "notes" → "note")
/// - No change if doesn't end with these patterns
///
/// # Arguments
/// * `s` - String to singularize
///
/// # Returns
/// Singularized string
fn singularize(s: &str) -> String {
  if s.ends_with("ies") && s.len() > 3 {
    format!("{}y", &s[..s.len() - 3])
  } else if s.ends_with('s') && s.len() > 1 {
    s[..s.len() - 1].to_string()
  } else {
    s.to_string()
  }
}

/// Resolves the type for a metadata entity, prioritizing frontmatter type over path inference.
///
/// # Resolution Priority
/// 1. If frontmatter has a "type" field (string), use that
/// 2. Otherwise, infer from path
///
/// # Arguments
/// * `path` - File path to use for inference if needed
/// * `frontmatter` - Frontmatter JSON to check for type field
///
/// # Returns
/// Resolved type as a String
///
/// # Examples
/// ```ignore
/// let path = Path::new("metadata/characters/hero.md");
/// let fm = json!({ "type": "protagonist" });
/// assert_eq!(resolve_type(path, &fm), "protagonist");
///
/// let fm_empty = json!({});
/// assert_eq!(resolve_type(path, &fm_empty), "character");
/// ```
#[must_use]
pub fn resolve_type(path: &Path, frontmatter: &Value) -> String {
  // Check frontmatter for explicit type
  if let Some(type_value) = frontmatter.get("type")
    && let Some(type_str) = type_value.as_str()
  {
    return type_str.to_string();
  }

  // Fall back to path inference
  infer_type_from_path(path)
}

/// Generates a namespace from a metadata file path relative to workspace root.
///
/// # Rules
/// - `metadata/` → "global"
/// - `book-01/metadata/` → "book-01"
/// - `book-01/part-01/metadata/` → "book-01/part-01"
/// - Extracts path segments between workspace root and metadata directory
///
/// # Arguments
/// * `metadata_path` - Full path to metadata file
/// * `workspace_root` - Path to workspace root
///
/// # Returns
/// Generated namespace as a String (or "global" if at workspace root)
///
/// # Examples
/// ```ignore
/// let metadata_path = Path::new("/project/book-01/metadata/characters.md");
/// let workspace_root = Path::new("/project");
/// assert_eq!(generate_namespace(metadata_path, workspace_root), "book-01");
/// ```
#[must_use]
pub fn generate_namespace(metadata_path: &Path, workspace_root: &Path) -> String {
  // Try to get relative path from workspace root to metadata file
  if let Ok(relative) = metadata_path.strip_prefix(workspace_root) {
    let components: Vec<&std::ffi::OsStr> = relative
      .components()
      .filter_map(|c| {
        if let std::path::Component::Normal(n) = c {
          Some(n)
        } else {
          None
        }
      })
      .collect();

    // Find "metadata" directory and extract path before it
    if let Some(metadata_idx) = components.iter().position(|c| c.to_string_lossy() == "metadata") {
      if metadata_idx == 0 {
        // metadata at root → "global"
        return "global".to_string();
      }

      // Join components before "metadata" with "/"
      let ns_parts: Vec<String> = components[..metadata_idx]
        .iter()
        .map(|c| c.to_string_lossy().to_string())
        .collect();

      return ns_parts.join("/");
    }
  }

  // Default: "global"
  "global".to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_infer_type_from_path_metadata_root() {
    let path = Path::new("metadata/config.md");
    assert_eq!(infer_type_from_path(path), "metadata");
  }

  #[test]
  fn test_infer_type_from_path_characters() {
    let path = Path::new("metadata/characters/hero.md");
    assert_eq!(infer_type_from_path(path), "character");
  }

  #[test]
  fn test_infer_type_from_path_scenes() {
    let path = Path::new("metadata/scenes/opening.md");
    assert_eq!(infer_type_from_path(path), "scene");
  }

  #[test]
  fn test_infer_type_from_path_singularize_notes() {
    let path = Path::new("metadata/notes/chapter1.md");
    assert_eq!(infer_type_from_path(path), "note");
  }

  #[test]
  fn test_infer_type_from_path_singularize_entries() {
    let path = Path::new("metadata/entries/plot.md");
    assert_eq!(infer_type_from_path(path), "entry");
  }

  #[test]
  fn test_singularize_ies_to_y() {
    assert_eq!(singularize("characters"), "character");
    assert_eq!(singularize("entries"), "entry");
    assert_eq!(singularize("stories"), "story");
  }

  #[test]
  fn test_singularize_es_to_empty() {
    assert_eq!(singularize("scenes"), "scene");
    assert_eq!(singularize("notes"), "note");
  }

  #[test]
  fn test_singularize_s_to_empty() {
    assert_eq!(singularize("things"), "thing");
    assert_eq!(singularize("items"), "item");
  }

  #[test]
  fn test_singularize_no_suffix() {
    assert_eq!(singularize("metadata"), "metadata");
    assert_eq!(singularize("data"), "data");
  }

  #[test]
  fn test_singularize_single_char() {
    assert_eq!(singularize("s"), "s");
    assert_eq!(singularize("a"), "a");
  }

  #[test]
  fn test_resolve_type_from_frontmatter() {
    let frontmatter = serde_json::json!({ "type": "protagonist" });
    let path = Path::new("metadata/characters/hero.md");

    assert_eq!(resolve_type(path, &frontmatter), "protagonist");
  }

  #[test]
  fn test_resolve_type_from_path_when_no_frontmatter_type() {
    let frontmatter = serde_json::json!({ "name": "Hero" });
    let path = Path::new("metadata/characters/hero.md");

    assert_eq!(resolve_type(path, &frontmatter), "character");
  }

  #[test]
  fn test_resolve_type_from_path_empty_frontmatter() {
    let frontmatter = serde_json::json!({});
    let path = Path::new("metadata/scenes/opening.md");

    assert_eq!(resolve_type(path, &frontmatter), "scene");
  }

  #[test]
  fn test_resolve_type_ignores_non_string_type() {
    let frontmatter = serde_json::json!({ "type": 123 });
    let path = Path::new("metadata/characters/hero.md");

    assert_eq!(resolve_type(path, &frontmatter), "character");
  }

  #[test]
  fn test_generate_namespace_at_metadata_root() {
    let metadata_path = Path::new("/project/metadata/config.md");
    let workspace_root = Path::new("/project");

    assert_eq!(generate_namespace(metadata_path, workspace_root), "global");
  }

  #[test]
  fn test_generate_namespace_one_level_deep() {
    let metadata_path = Path::new("/project/book-01/metadata/characters.md");
    let workspace_root = Path::new("/project");

    assert_eq!(generate_namespace(metadata_path, workspace_root), "book-01");
  }

  #[test]
  fn test_generate_namespace_two_levels_deep() {
    let metadata_path = Path::new("/project/book-01/part-01/metadata/scene.md");
    let workspace_root = Path::new("/project");

    assert_eq!(generate_namespace(metadata_path, workspace_root), "book-01/part-01");
  }

  #[test]
  fn test_generate_namespace_relative_paths() {
    let metadata_path = Path::new("metadata/config.md");
    let workspace_root = Path::new(".");

    assert_eq!(generate_namespace(metadata_path, workspace_root), "global");
  }

  #[test]
  fn test_generate_namespace_three_levels_deep() {
    let metadata_path = Path::new("/project/series-01/book-02/chapter-05/metadata/data.md");
    let workspace_root = Path::new("/project");

    assert_eq!(
      generate_namespace(metadata_path, workspace_root),
      "series-01/book-02/chapter-05"
    );
  }
}
