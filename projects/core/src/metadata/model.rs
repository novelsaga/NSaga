use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

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
  #[must_use]
  pub fn new(
    id: impl Into<String>,
    type_: impl Into<String>,
    namespace: impl Into<String>,
    frontmatter: Value,
    body: impl Into<String>,
  ) -> Self {
    Self {
      id: id.into(),
      type_: type_.into(),
      namespace: namespace.into(),
      frontmatter,
      body: body.into(),
    }
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
  fn export_bindings() {
    // Only run when called from xtask (TS_RS_EXPORT_DIR is set)
    if std::env::var("TS_RS_EXPORT_DIR").is_err() {
      println!("Skipping export_bindings - only runs via xtask gen-ts-bindings");
      return;
    }
    MetadataEntity::export().expect("failed to export MetadataEntity");
  }
}
