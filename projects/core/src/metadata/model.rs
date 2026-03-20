use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

use crate::{
  document::MarkdownParts,
  metadata::parser::{generate_namespace, resolve_type},
};

/// Represents the parts needed to construct a `MetadataEntity`.
#[derive(Debug, Clone)]
pub struct MetadataEntityParts {
  /// Unique identifier for the metadata entity
  pub id: String,
  /// Type of the metadata entity
  pub type_: String,
  /// Namespace grouping for the entity
  pub namespace: String,
  /// Frontmatter data stored as JSON value
  pub frontmatter: Value,
  /// Body content as string
  pub body: String,
}

/// Represents a metadata entity with id, type, namespace, frontmatter, and body.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export_to = "_metadata.ts")]
pub struct MetadataEntity {
  /// Unique identifier for the metadata entity
  pub id: String,
  /// Type of the metadata entity (renamed from type_ in JSON)
  #[ts(rename = "type")]
  pub type_: String,
  /// Namespace grouping for the entity
  pub namespace: String,
  /// Frontmatter data stored as JSON value
  /// Frontmatter data stored as JSON value
  #[ts(type = "Record<string, any>")]
  pub frontmatter: Value,
  /// Body content as string
  pub body: String,
}

impl MetadataEntity {
  /// Creates a new `MetadataEntity` with the given parameters.
  ///
  /// # Arguments
  /// * `id` - Unique identifier
  /// * `type_` - Type classification
  /// * `namespace` - Namespace grouping
  /// * `frontmatter` - Frontmatter data
  /// * `body` - Body content
  ///
  /// Creates a new `MetadataEntity` from `MetadataEntityParts`.
  ///
  /// # Arguments
  /// * `parts` - Struct containing id, type_, namespace, frontmatter, and body
  ///
  /// # Returns
  /// * `MetadataEntity` - Fully constructed entity
  #[must_use]
  pub fn from_parts(parts: MetadataEntityParts) -> Self {
    Self {
      id: parts.id,
      type_: parts.type_,
      namespace: parts.namespace,
      frontmatter: parts.frontmatter,
      body: parts.body,
    }
  }

  /// Creates a new `MetadataEntity` with the given parameters.
  ///
  /// # Arguments
  /// * `id` - Unique identifier
  /// * `type_` - Type classification
  /// * `namespace` - Namespace grouping
  /// * `frontmatter` - Frontmatter data
  /// * `body` - Body content
  #[must_use]
  pub fn new(
    id: impl Into<String>,
    type_: impl Into<String>,
    namespace: impl Into<String>,
    frontmatter: Value,
    body: impl Into<String>,
  ) -> Self {
    Self::from_parts(MetadataEntityParts {
      id: id.into(),
      type_: type_.into(),
      namespace: namespace.into(),
      frontmatter,
      body: body.into(),
    })
  }

  /// Gets a field value from the frontmatter by key.
  ///
  /// # Arguments
  /// * `key` - The field key to retrieve
  ///
  /// # Returns
  /// * `Some(&Value)` if the key exists in frontmatter
  /// * `None` if the key does not exist or frontmatter is not an object
  #[must_use]
  pub fn get_field(&self, key: &str) -> Option<&Value> {
    self.frontmatter.get(key)
  }

  /// Extracts the type from the frontmatter "type" field.
  ///
  /// # Returns
  /// * `Some(String)` if "type" field exists and is a string in frontmatter
  /// * `None` if "type" field does not exist or is not a string
  #[must_use]
  pub fn get_type_from_frontmatter(&self) -> Option<String> {
    self
      .frontmatter
      .get("type")
      .and_then(|v| v.as_str())
      .map(ToString::to_string)
  }

  #[must_use]
  pub fn to_hover_markdown(&self) -> String {
    let title = ["name", "title"]
      .into_iter()
      .find_map(|key| self.get_field(key).and_then(|value| value.as_str()))
      .unwrap_or(&self.id);

    let mut sections = vec![
      format!("# {title}"),
      format!("- Type: `{}`", self.type_),
      format!("- Namespace: `{}`", self.namespace),
      format!("- ID: `{}`", self.id),
    ];

    if !self.body.trim().is_empty() {
      sections.push(String::new());
      sections.push(self.body.trim().to_string());
    }

    sections.join("\n")
  }
}

impl TryFrom<(MarkdownParts, &Path, &Path)> for MetadataEntity {
  type Error = String;

  /// Create `MetadataEntity` from markdown parts, file path, and workspace root.
  ///
  /// This is a convenience wrapper that constructs `MetadataEntityParts` and calls `from_parts()`.
  ///
  /// # Arguments
  /// * `markdown_parts` - Parsed markdown with frontmatter and body
  /// * `file_path` - Full path to the metadata file
  /// * `workspace_root` - Path to the workspace root
  ///
  /// # Returns
  /// * `Ok(MetadataEntity)` on success
  /// * `Err(String)` if entity construction fails
  fn try_from(value: (MarkdownParts, &Path, &Path)) -> Result<Self, Self::Error> {
    let (markdown_parts, file_path, workspace_root) = value;

    // Generate ID from filename (without extension)
    let id = file_path
      .file_stem()
      .and_then(|stem| stem.to_str())
      .map(ToString::to_string)
      .ok_or_else(|| "Failed to extract filename for ID".to_string())?;

    // Resolve type: frontmatter takes priority
    let type_ = resolve_type(file_path, &markdown_parts.frontmatter);

    // Generate namespace from file location
    let namespace = generate_namespace(file_path, workspace_root);

    Ok(Self::from_parts(MetadataEntityParts {
      id,
      type_,
      namespace,
      frontmatter: markdown_parts.frontmatter,
      body: markdown_parts.body,
    }))
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_new_creates_entity() {
    let frontmatter = json!({ "author": "John Doe" });
    let entity = MetadataEntity::new("entity-1", "article", "blog", frontmatter, "This is the body");

    assert_eq!(entity.id, "entity-1");
    assert_eq!(entity.type_, "article");
    assert_eq!(entity.namespace, "blog");
    assert_eq!(entity.body, "This is the body");
  }

  #[test]
  fn test_get_field_returns_value() {
    let frontmatter = json!({
        "author": "Jane Doe",
        "tags": ["rust", "metadata"]
    });
    let entity = MetadataEntity::new("id", "type", "ns", frontmatter, "body");

    assert_eq!(entity.get_field("author"), Some(&json!("Jane Doe")));
    assert_eq!(entity.get_field("tags"), Some(&json!(["rust", "metadata"])));
  }

  #[test]
  fn test_get_field_returns_none_for_missing_key() {
    let frontmatter = json!({ "author": "John" });
    let entity = MetadataEntity::new("id", "type", "ns", frontmatter, "body");

    assert_eq!(entity.get_field("missing"), None);
  }

  #[test]
  fn test_get_type_from_frontmatter_success() {
    let frontmatter = json!({ "type": "article" });
    let entity = MetadataEntity::new("id", "entity-type", "ns", frontmatter, "body");

    assert_eq!(entity.get_type_from_frontmatter(), Some("article".to_string()));
  }

  #[test]
  fn test_get_type_from_frontmatter_missing() {
    let frontmatter = json!({ "author": "John" });
    let entity = MetadataEntity::new("id", "type", "ns", frontmatter, "body");

    assert_eq!(entity.get_type_from_frontmatter(), None);
  }

  #[test]
  fn test_get_type_from_frontmatter_non_string() {
    let frontmatter = json!({ "type": 123 });
    let entity = MetadataEntity::new("id", "type", "ns", frontmatter, "body");

    assert_eq!(entity.get_type_from_frontmatter(), None);
  }

  #[test]
  fn test_metadata_entity_serialization() {
    let frontmatter = json!({ "key": "value" });
    let entity = MetadataEntity::new("id1", "document", "notes", frontmatter, "body text");

    let serialized = serde_json::to_string(&entity).expect("serialization failed");
    let deserialized: MetadataEntity = serde_json::from_str(&serialized).expect("deserialization failed");

    assert_eq!(entity, deserialized);
  }
  #[test]
  fn test_from_parts_creates_entity() {
    let markdown_content = "---\nauthor: John\n---\nA brave hero";
    let parts = crate::document::MarkdownParts::parse(markdown_content);

    let file_path = Path::new("/project/metadata/characters/entity-1.md");
    let workspace_root = Path::new("/project");

    // Use TryFrom to construct entity from markdown parts
    let entity = MetadataEntity::try_from((parts, file_path, workspace_root)).unwrap();

    assert_eq!(entity.id, "entity-1");
    assert_eq!(entity.type_, "character");
    assert_eq!(entity.namespace, "global");
    assert_eq!(entity.body, "A brave hero");
    assert_eq!(entity.get_field("author"), Some(&json!("John")));
  }

  #[test]
  fn test_from_parts_and_new_equivalent() {
    let frontmatter = json!({ "title": "Test" });

    let entity1 = MetadataEntity::new("id1", "article", "global", frontmatter.clone(), "content");

    // Construct entity using from_parts directly
    let parts = MetadataEntityParts {
      id: "id1".to_string(),
      type_: "article".to_string(),
      namespace: "global".to_string(),
      frontmatter: frontmatter.clone(),
      body: "content".to_string(),
    };
    let entity2 = MetadataEntity::from_parts(parts);

    // Both should have the same fields
    assert_eq!(entity1.id, entity2.id);
    assert_eq!(entity1.frontmatter, entity2.frontmatter);
    assert_eq!(entity1.body, entity2.body);
  }

  #[test]
  fn test_try_from_markdown_parts_success() {
    let markdown_content = "---\ntitle: Hero\ntype: character\n---\n# Character Details\nBrave and noble.";
    let parts = crate::document::MarkdownParts::parse(markdown_content);

    let file_path = Path::new("/project/book-01/metadata/characters/hero.md");
    let workspace_root = Path::new("/project");

    let result = MetadataEntity::try_from((parts, file_path, workspace_root));
    assert!(result.is_ok());

    let entity = result.unwrap();
    assert_eq!(entity.id, "hero");
    assert_eq!(entity.type_, "character"); // From frontmatter
    assert_eq!(entity.namespace, "book-01");
    assert_eq!(entity.body, "# Character Details\nBrave and noble.");
    assert_eq!(entity.get_field("title"), Some(&json!("Hero")));
  }

  #[test]
  fn test_try_from_markdown_parts_infers_type_from_path() {
    let markdown_content = "---\ntitle: Scene\n---\n## Opening scene";
    let parts = crate::document::MarkdownParts::parse(markdown_content);

    let file_path = Path::new("/project/book-01/part-01/metadata/scenes/opening.md");
    let workspace_root = Path::new("/project");

    let entity = MetadataEntity::try_from((parts, file_path, workspace_root)).unwrap();
    assert_eq!(entity.id, "opening");
    assert_eq!(entity.type_, "scene"); // Inferred from path
    assert_eq!(entity.namespace, "book-01/part-01");
  }

  #[test]
  fn test_try_from_metadata_at_root() {
    let markdown_content = "---\nstatus: published\n---\nGlobal metadata";
    let parts = crate::document::MarkdownParts::parse(markdown_content);

    let file_path = Path::new("/project/metadata/settings.md");
    let workspace_root = Path::new("/project");

    let entity = MetadataEntity::try_from((parts, file_path, workspace_root)).unwrap();
    assert_eq!(entity.id, "settings");
    assert_eq!(entity.namespace, "global");
    assert_eq!(entity.body, "Global metadata");
  }

  #[test]
  fn test_try_from_frontmatter_type_priority() {
    // Even though path suggests "scene", frontmatter type should take priority
    let markdown_content = "---\ntype: event\n---\nSpecial event";
    let parts = crate::document::MarkdownParts::parse(markdown_content);

    let file_path = Path::new("/project/metadata/scenes/episode.md");
    let workspace_root = Path::new("/project");

    let entity = MetadataEntity::try_from((parts, file_path, workspace_root)).unwrap();
    assert_eq!(entity.type_, "event"); // Frontmatter priority
  }

  #[test]
  fn export_bindings() {
    // Only run when called from xtask (TS_RS_EXPORT_DIR is set)
    if std::env::var("TS_RS_EXPORT_DIR").is_err() {
      println!("Skipping export_bindings - only runs via xtask gen-ts-bindings");
      return;
    }
    MetadataEntity::export().expect("failed to export MetadataEntity");
  }

  #[test]
  fn to_hover_markdown_prefers_title_and_includes_metadata_fields() {
    let entity = MetadataEntity::new(
      "hero-alpha",
      "character",
      "cast",
      json!({ "title": "Hero Alpha" }),
      "Brave hero body.",
    );

    let markdown = entity.to_hover_markdown();

    assert!(markdown.contains("# Hero Alpha"));
    assert!(markdown.contains("- Type: `character`"));
    assert!(markdown.contains("- Namespace: `cast`"));
    assert!(markdown.contains("- ID: `hero-alpha`"));
    assert!(markdown.contains("Brave hero body."));
  }
}
