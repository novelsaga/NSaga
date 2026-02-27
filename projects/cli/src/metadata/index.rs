use std::path::Path;

use blake3;
use novelsaga_core::metadata::model::MetadataEntity;
use serde_json;
use sled::Db;

/// `IndexManager` provides persistent key-value storage for metadata entities
/// with secondary indexes for name, type, and namespace lookups.
#[allow(dead_code)]
pub struct IndexManager {
  db: Db,
}

#[allow(dead_code)]
impl IndexManager {
  /// Opens or creates a sled database at the given path.
  ///
  /// # Arguments
  /// * `path` - Path to the sled database directory
  ///
  /// # Returns
  /// * `Ok(IndexManager)` on success
  /// * `Err(sled::Error)` if database cannot be opened
  pub fn open(path: &Path) -> Result<Self, sled::Error> {
    let db = sled::open(path)?;
    Ok(IndexManager { db })
  }

  /// Generates a unique 16-character ID from a source string using blake3 hash.
  ///
  /// # Arguments
  /// * `source` - Source string to hash
  ///
  /// # Returns
  /// * 16-character hexadecimal hash (consistent for same input)
  pub fn generate_id(source: &str) -> String {
    let hash = blake3::hash(source.as_bytes());
    hash.to_hex()[..16].to_string()
  }

  /// Indexes a metadata entity by storing it and updating secondary indexes.
  ///
  /// # Arguments
  /// * `entity` - The `MetadataEntity` to index
  ///
  /// # Returns
  /// * `Ok(())` on success
  /// * `Err(sled::Error)` if database operation fails
  pub fn index_entity(&self, entity: &MetadataEntity) -> Result<(), sled::Error> {
    // Clean up old indexes if entity already exists (prevents ghost indexes)
    if self.get_by_id(&entity.id)?.is_some() {
      self.remove_entity(&entity.id)?;
    }

    // Serialize entity

    let entity_bytes = serde_json::to_vec(entity)
      .map_err(|e| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())))?;

    // Store entity: entity:{id}
    let entity_key = format!("entity:{}", entity.id);
    self.db.insert(entity_key.as_bytes(), entity_bytes.as_slice())?;

    // Update name index: name:{namespace}:{id}
    let name_key = format!("name:{}:{}", entity.namespace, entity.id);
    self.db.insert(name_key.as_bytes(), entity.id.as_bytes())?;

    // Update type index: type:{type}:{id}
    let type_key = format!("type:{}:{}", entity.type_, entity.id);
    self.db.insert(type_key.as_bytes(), entity.id.as_bytes())?;

    // Update namespace index: ns:{namespace}:{id}
    let ns_key = format!("ns:{}:{}", entity.namespace, entity.id);
    self.db.insert(ns_key.as_bytes(), entity.id.as_bytes())?;

    Ok(())
  }

  /// Retrieves an entity by its ID.
  ///
  /// # Arguments
  /// * `id` - The entity ID to retrieve
  ///
  /// # Returns
  /// * `Ok(Some(entity))` if found
  /// * `Ok(None)` if not found
  /// * `Err(sled::Error)` if database operation fails
  pub fn get_by_id(&self, id: &str) -> Result<Option<MetadataEntity>, sled::Error> {
    let key = format!("entity:{id}");
    match self.db.get(key.as_bytes())? {
      Some(bytes) => {
        let entity = serde_json::from_slice(&bytes)
          .map_err(|e| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())))?;
        Ok(Some(entity))
      }
      None => Ok(None),
    }
  }

  /// Lists all entities with a specific type.
  ///
  /// # Arguments
  /// * `type_` - The type to filter by
  ///
  /// # Returns
  /// * `Ok(Vec<MetadataEntity>)` - Vector of matching entities
  /// * `Err(sled::Error)` if database operation fails
  pub fn list_by_type(&self, type_: &str) -> Result<Vec<MetadataEntity>, sled::Error> {
    let prefix = format!("type:{type_}:");
    self.collect_entities_by_prefix(&prefix)
  }

  /// Lists all entities with a specific namespace.
  ///
  /// # Arguments
  /// * `namespace` - The namespace to filter by
  ///
  /// # Returns
  /// * `Ok(Vec<MetadataEntity>)` - Vector of matching entities
  /// * `Err(sled::Error)` if database operation fails
  pub fn list_by_namespace(&self, namespace: &str) -> Result<Vec<MetadataEntity>, sled::Error> {
    let prefix = format!("ns:{namespace}:");
    self.collect_entities_by_prefix(&prefix)
  }

  /// Removes an entity and all its index entries.
  ///
  /// # Arguments
  /// * `id` - The entity ID to remove
  ///
  /// # Returns
  /// * `Ok(())` on success
  /// * `Err(sled::Error)` if database operation fails
  pub fn remove_entity(&self, id: &str) -> Result<(), sled::Error> {
    // Get entity first to find namespace and type for index cleanup
    if let Some(entity) = self.get_by_id(id)? {
      // Remove entity: entity:{id}
      let entity_key = format!("entity:{id}");
      self.db.remove(entity_key.as_bytes())?;

      // Remove name index: name:{namespace}:{id}
      let name_key = format!("name:{}:{}", entity.namespace, id);
      self.db.remove(name_key.as_bytes())?;

      // Remove type index: type:{type}:{id}
      let type_key = format!("type:{}:{}", entity.type_, id);
      self.db.remove(type_key.as_bytes())?;

      // Remove namespace index: ns:{namespace}:{id}
      let ns_key = format!("ns:{}:{}", entity.namespace, id);
      self.db.remove(ns_key.as_bytes())?;
    }

    Ok(())
  }

  /// Rebuilds all indexes from scratch by scanning all entities.
  ///
  /// # Returns
  /// * `Ok(())` on success
  /// * `Err(sled::Error)` if database operation fails
  pub fn rebuild(&self) -> Result<(), sled::Error> {
    // Collect all entities
    let mut entities = Vec::new();
    for item in self.db.scan_prefix(b"entity:") {
      let (_key, bytes) = item?;
      if let Ok(entity) = serde_json::from_slice::<MetadataEntity>(&bytes) {
        entities.push(entity);
      }
    }

    // Clear all index entries (keep entity entries)
    let prefixes = vec!["name:", "type:", "ns:"];
    for prefix in prefixes {
      let mut to_remove = Vec::new();
      for item in self.db.scan_prefix(prefix.as_bytes()) {
        let (key, _) = item?;
        to_remove.push(key.to_vec());
      }
      for key in to_remove {
        self.db.remove(&key)?;
      }
    }

    // Re-index all entities
    for entity in entities {
      self.index_entity(&entity)?;
    }

    Ok(())
  }

  /// Flushes the database to ensure all writes are persisted.
  ///
  /// # Returns
  /// * `Ok(())` on success
  /// * `Err(sled::Error)` if database operation fails
  pub fn flush(&self) -> Result<(), sled::Error> {
    let _ = self.db.flush()?;
    Ok(())
  }

  /// Lists all entities by scanning the entity: prefix.
  ///
  /// # Returns
  /// * `Ok(Vec<MetadataEntity>)` - Vector of all entities
  /// * `Err(sled::Error)` if database operation fails
  pub fn list_all(&self) -> Result<Vec<MetadataEntity>, sled::Error> {
    let prefix = "entity:";
    self.collect_entities_by_prefix(prefix)
  }

  /// Helper function to collect entities from a prefix scan.
  fn collect_entities_by_prefix(&self, prefix: &str) -> Result<Vec<MetadataEntity>, sled::Error> {
    let mut entities = Vec::new();

    for item in self.db.scan_prefix(prefix.as_bytes()) {
      let (_key, id_bytes) = item?;
      let id = String::from_utf8(id_bytes.to_vec())
        .map_err(|e| sled::Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())))?;

      if let Some(entity) = self.get_by_id(&id)? {
        entities.push(entity);
      }
    }

    Ok(entities)
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;
  use tempfile::TempDir;

  use super::*;

  #[test]
  fn test_generate_id() {
    // Same source should produce same ID
    let id1 = IndexManager::generate_id("test-source");
    let id2 = IndexManager::generate_id("test-source");
    assert_eq!(id1, id2);

    // Different sources should produce different IDs
    let id3 = IndexManager::generate_id("different-source");
    assert_ne!(id1, id3);

    // ID should be exactly 16 characters
    assert_eq!(id1.len(), 16);
    assert_eq!(id3.len(), 16);

    // ID should be valid hex
    assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));
  }

  #[test]
  fn test_index_manager_basic_crud() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = IndexManager::open(temp_dir.path())?;

    // Create an entity
    let entity = MetadataEntity::new(
      "test-id-123",
      "article",
      "blog",
      json!({"author": "John Doe"}),
      "This is the body",
    );

    // Index the entity
    manager.index_entity(&entity)?;

    // Retrieve by ID
    let retrieved = manager.get_by_id("test-id-123")?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.as_ref().unwrap().id, "test-id-123");
    assert_eq!(retrieved.as_ref().unwrap().type_, "article");

    // Remove the entity
    manager.remove_entity("test-id-123")?;

    // Verify it's gone
    let removed = manager.get_by_id("test-id-123")?;
    assert!(removed.is_none());

    Ok(())
  }

  #[test]
  fn test_list_by_type() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = IndexManager::open(temp_dir.path())?;

    // Create multiple entities of same type
    let entity1 = MetadataEntity::new("id-1", "article", "blog", json!({}), "body1");
    let entity2 = MetadataEntity::new("id-2", "article", "blog", json!({}), "body2");
    let entity3 = MetadataEntity::new("id-3", "comment", "blog", json!({}), "body3");

    manager.index_entity(&entity1)?;
    manager.index_entity(&entity2)?;
    manager.index_entity(&entity3)?;

    // List by type
    let articles = manager.list_by_type("article")?;
    assert_eq!(articles.len(), 2);

    let comments = manager.list_by_type("comment")?;
    assert_eq!(comments.len(), 1);

    Ok(())
  }

  #[test]
  fn test_list_by_namespace() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = IndexManager::open(temp_dir.path())?;

    // Create entities in different namespaces
    let entity1 = MetadataEntity::new("id-1", "article", "blog", json!({}), "body1");
    let entity2 = MetadataEntity::new("id-2", "article", "docs", json!({}), "body2");

    manager.index_entity(&entity1)?;
    manager.index_entity(&entity2)?;

    // List by namespace
    let blog_entities = manager.list_by_namespace("blog")?;
    assert_eq!(blog_entities.len(), 1);
    assert_eq!(blog_entities[0].namespace, "blog");

    let docs_entities = manager.list_by_namespace("docs")?;
    assert_eq!(docs_entities.len(), 1);
    assert_eq!(docs_entities[0].namespace, "docs");

    Ok(())
  }

  #[test]
  fn test_rebuild_indexes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = IndexManager::open(temp_dir.path())?;

    // Create some entities
    let entity1 = MetadataEntity::new("id-1", "article", "blog", json!({}), "body1");
    let entity2 = MetadataEntity::new("id-2", "article", "blog", json!({}), "body2");

    manager.index_entity(&entity1)?;
    manager.index_entity(&entity2)?;

    // Rebuild indexes
    manager.rebuild()?;

    // Verify entities are still accessible
    let articles = manager.list_by_type("article")?;
    assert_eq!(articles.len(), 2);

    Ok(())
  }

  #[test]
  fn test_flush() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = IndexManager::open(temp_dir.path())?;

    let entity = MetadataEntity::new("id-1", "article", "blog", json!({}), "body");

    manager.index_entity(&entity)?;
    manager.flush()?;

    // After flush, data should be persistent
    let retrieved = manager.get_by_id("id-1")?;
    assert!(retrieved.is_some());

    Ok(())
  }

  #[test]
  fn test_ghost_index_cleanup_on_update() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let manager = IndexManager::open(temp_dir.path())?;

    // Create and index entity with namespace "blog"
    let entity1 = MetadataEntity::new("id-1", "article", "blog", json!({}), "body1");
    manager.index_entity(&entity1)?;

    // Verify it's in "blog" namespace
    let blog_entities = manager.list_by_namespace("blog")?;
    assert_eq!(blog_entities.len(), 1);
    assert_eq!(blog_entities[0].id, "id-1");

    // Update same entity with different namespace
    let entity2 = MetadataEntity::new("id-1", "article", "docs", json!({}), "body1");
    manager.index_entity(&entity2)?;

    // Verify entity moved to "docs" namespace
    let docs_entities = manager.list_by_namespace("docs")?;
    assert_eq!(docs_entities.len(), 1);
    assert_eq!(docs_entities[0].namespace, "docs");

    // Verify ghost index is cleaned: "blog" should now be empty
    let blog_entities_after = manager.list_by_namespace("blog")?;
    assert_eq!(
      blog_entities_after.len(),
      0,
      "Ghost index not cleaned: entity still found in old namespace"
    );

    Ok(())
  }
}
