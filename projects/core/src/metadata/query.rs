use std::collections::HashMap;

/// Metadata query module
///
/// Provides query interfaces for metadata entities
use crate::metadata::MetadataEntity;

/// Result of a query operation
#[derive(Debug, Clone)]
pub struct QueryResult {
  /// Collection of metadata entities matching the query
  pub entities: Vec<MetadataEntity>,
  /// Total number of entities in the result
  pub total: usize,
}

impl QueryResult {
  /// Creates a new query result
  #[must_use]
  pub fn new(entities: Vec<MetadataEntity>) -> Self {
    let total = entities.len();
    Self { entities, total }
  }
}

/// Trait for querying metadata entities
pub trait MetadataQuery {
  /// Retrieves a metadata entity by its ID
  ///
  /// # Arguments
  /// * `id` - The entity ID to look up
  ///
  /// # Returns
  /// * `Some(MetadataEntity)` if found
  /// * `None` if not found
  fn get_by_id(&self, id: &str) -> Option<MetadataEntity>;

  /// Retrieves a metadata entity by name and namespace
  ///
  /// # Arguments
  /// * `name` - The entity name
  /// * `namespace` - The namespace to search in
  ///
  /// # Returns
  /// * `Some(MetadataEntity)` if found
  /// * `None` if not found
  fn get_by_name(&self, name: &str, namespace: &str) -> Option<MetadataEntity>;

  /// Lists all metadata entities of a given type
  ///
  /// # Arguments
  /// * `type_` - The entity type to filter by
  /// * `namespace` - Optional namespace filter
  ///
  /// # Returns
  /// A vector of matching entities
  fn list_by_type(&self, type_: &str, namespace: Option<&str>) -> Vec<MetadataEntity>;

  /// Lists all metadata entities in a given namespace
  ///
  /// # Arguments
  /// * `namespace` - The namespace to list
  ///
  /// # Returns
  /// A vector of all entities in the namespace
  fn list_by_namespace(&self, namespace: &str) -> Vec<MetadataEntity>;

  /// Searches for metadata entities by content and optional type filter
  ///
  /// # Arguments
  /// * `query` - Search string to match against entity IDs, names, and body content
  /// * `type_filter` - Optional type to filter by
  ///
  /// # Returns
  /// A vector of matching entities
  fn search(&self, query: &str, type_filter: Option<&str>) -> Vec<MetadataEntity>;
}

/// In-memory implementation of `MetadataQuery`
#[derive(Debug, Clone)]
pub struct InMemoryMetadataStore {
  /// Primary store: id -> entity
  entities: HashMap<String, MetadataEntity>,
  /// Secondary index: name:namespace -> id
  by_name: HashMap<String, String>,
}

impl InMemoryMetadataStore {
  /// Creates a new empty metadata store
  #[must_use]
  pub fn new() -> Self {
    Self {
      entities: HashMap::new(),
      by_name: HashMap::new(),
    }
  }

  /// Inserts or updates a metadata entity in the store
  ///
  /// # Arguments
  /// * `entity` - The entity to insert
  pub fn insert(&mut self, entity: MetadataEntity) {
    let name_key = format!("{}:{}", entity.id, entity.namespace);
    self.by_name.insert(name_key, entity.id.clone());
    self.entities.insert(entity.id.clone(), entity);
  }

  /// Returns the number of entities in the store
  #[must_use]
  pub fn len(&self) -> usize {
    self.entities.len()
  }

  /// Checks if the store is empty
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.entities.is_empty()
  }
}

impl Default for InMemoryMetadataStore {
  fn default() -> Self {
    Self::new()
  }
}

impl MetadataQuery for InMemoryMetadataStore {
  fn get_by_id(&self, id: &str) -> Option<MetadataEntity> {
    self.entities.get(id).cloned()
  }

  fn get_by_name(&self, name: &str, namespace: &str) -> Option<MetadataEntity> {
    let name_key = format!("{name}:{namespace}");
    self
      .by_name
      .get(&name_key)
      .and_then(|id| self.entities.get(id).cloned())
  }

  fn list_by_type(&self, type_: &str, namespace: Option<&str>) -> Vec<MetadataEntity> {
    self
      .entities
      .values()
      .filter(|entity| entity.type_ == type_ && namespace.is_none_or(|ns| entity.namespace == ns))
      .cloned()
      .collect()
  }

  fn list_by_namespace(&self, namespace: &str) -> Vec<MetadataEntity> {
    self
      .entities
      .values()
      .filter(|entity| entity.namespace == namespace)
      .cloned()
      .collect()
  }

  fn search(&self, query: &str, type_filter: Option<&str>) -> Vec<MetadataEntity> {
    let query_lower = query.to_lowercase();
    self
      .entities
      .values()
      .filter(|entity| {
        let matches_type = type_filter.is_none_or(|t| entity.type_ == t);
        let matches_content = entity.id.to_lowercase().contains(&query_lower)
          || entity.namespace.to_lowercase().contains(&query_lower)
          || entity.body.to_lowercase().contains(&query_lower);
        matches_type && matches_content
      })
      .cloned()
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  #[test]
  fn test_query_result_creation() {
    let entities = vec![];
    let result = QueryResult::new(entities);
    assert_eq!(result.total, 0);
    assert!(result.entities.is_empty());
  }

  #[test]
  fn test_query_result_with_entities() {
    let entity = MetadataEntity::new("test-1", "article", "blog", json!({}), "Test body");
    let result = QueryResult::new(vec![entity]);
    assert_eq!(result.total, 1);
    assert_eq!(result.entities.len(), 1);
  }

  #[test]
  fn test_in_memory_store_new() {
    let store = InMemoryMetadataStore::new();
    assert!(store.is_empty());
    assert_eq!(store.len(), 0);
  }

  #[test]
  fn test_in_memory_store_insert() {
    let mut store = InMemoryMetadataStore::new();
    let entity = MetadataEntity::new("id-1", "document", "notes", json!({}), "body");

    store.insert(entity);
    assert_eq!(store.len(), 1);
    assert!(!store.is_empty());
  }

  #[test]
  fn test_get_by_id() {
    let mut store = InMemoryMetadataStore::new();
    let entity = MetadataEntity::new("test-id", "article", "blog", json!({}), "Test content");
    store.insert(entity.clone());

    let result = store.get_by_id("test-id");
    assert_eq!(result, Some(entity));
  }

  #[test]
  fn test_get_by_id_not_found() {
    let store = InMemoryMetadataStore::new();
    let result = store.get_by_id("nonexistent");
    assert_eq!(result, None);
  }

  #[test]
  fn test_get_by_name() {
    let mut store = InMemoryMetadataStore::new();
    let entity = MetadataEntity::new("id-1", "document", "docs", json!({}), "content");
    store.insert(entity.clone());

    let result = store.get_by_name("id-1", "docs");
    assert_eq!(result, Some(entity));
  }

  #[test]
  fn test_get_by_name_not_found() {
    let mut store = InMemoryMetadataStore::new();
    let entity = MetadataEntity::new("id-1", "document", "docs", json!({}), "content");
    store.insert(entity);

    let result = store.get_by_name("id-1", "wrong-ns");
    assert_eq!(result, None);
  }

  #[test]
  fn test_list_by_type() {
    let mut store = InMemoryMetadataStore::new();
    store.insert(MetadataEntity::new("id-1", "article", "blog", json!({}), "content 1"));
    store.insert(MetadataEntity::new("id-2", "article", "blog", json!({}), "content 2"));
    store.insert(MetadataEntity::new("id-3", "page", "blog", json!({}), "content 3"));

    let results = store.list_by_type("article", None);
    assert_eq!(results.len(), 2);
  }

  #[test]
  fn test_list_by_type_with_namespace() {
    let mut store = InMemoryMetadataStore::new();
    store.insert(MetadataEntity::new("id-1", "article", "blog", json!({}), "content 1"));
    store.insert(MetadataEntity::new("id-2", "article", "docs", json!({}), "content 2"));

    let results = store.list_by_type("article", Some("blog"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "id-1");
  }

  #[test]
  fn test_list_by_namespace() {
    let mut store = InMemoryMetadataStore::new();
    store.insert(MetadataEntity::new("id-1", "article", "blog", json!({}), "content 1"));
    store.insert(MetadataEntity::new("id-2", "page", "blog", json!({}), "content 2"));
    store.insert(MetadataEntity::new("id-3", "article", "docs", json!({}), "content 3"));

    let results = store.list_by_namespace("blog");
    assert_eq!(results.len(), 2);
  }

  #[test]
  fn test_search_by_id() {
    let mut store = InMemoryMetadataStore::new();
    store.insert(MetadataEntity::new(
      "search-test-id",
      "article",
      "blog",
      json!({}),
      "Some content",
    ));

    let results = store.search("search-test", None);
    assert_eq!(results.len(), 1);
  }

  #[test]
  fn test_search_by_body() {
    let mut store = InMemoryMetadataStore::new();
    store.insert(MetadataEntity::new(
      "id-1",
      "article",
      "blog",
      json!({}),
      "This is searchable content",
    ));

    let results = store.search("searchable", None);
    assert_eq!(results.len(), 1);
  }

  #[test]
  fn test_search_case_insensitive() {
    let mut store = InMemoryMetadataStore::new();
    store.insert(MetadataEntity::new("MyId", "article", "blog", json!({}), "MyBody"));

    let results = store.search("myid", None);
    assert_eq!(results.len(), 1);
  }

  #[test]
  fn test_search_with_type_filter() {
    let mut store = InMemoryMetadataStore::new();
    store.insert(MetadataEntity::new(
      "id-1",
      "article",
      "blog",
      json!({}),
      "searchable content",
    ));
    store.insert(MetadataEntity::new(
      "id-2",
      "page",
      "blog",
      json!({}),
      "searchable content",
    ));

    let results = store.search("searchable", Some("article"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].type_, "article");
  }

  #[test]
  fn test_search_no_results() {
    let store = InMemoryMetadataStore::new();
    let results = store.search("nonexistent", None);
    assert_eq!(results.len(), 0);
  }
}
