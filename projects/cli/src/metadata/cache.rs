use std::sync::Arc;

use moka::future::Cache;
use novelsaga_core::metadata::model::MetadataEntity;

/// Cache manager for metadata entities using moka async cache
#[allow(dead_code)]
pub struct CacheManager {
  cache: Cache<String, Arc<MetadataEntity>>,
}

#[allow(dead_code)]
impl CacheManager {
  /// Create a new `CacheManager` with specified capacity
  pub fn new(capacity: u64) -> Self {
    let cache = Cache::builder().max_capacity(capacity).build();
    Self { cache }
  }

  /// Get a metadata entity from cache
  pub async fn get(&self, id: &str) -> Option<Arc<MetadataEntity>> {
    self.cache.get(id).await
  }

  /// Insert a metadata entity into cache
  pub async fn insert(&self, id: String, entity: MetadataEntity) {
    self.cache.insert(id, Arc::new(entity)).await;
  }

  /// Insert multiple metadata entities into cache
  pub async fn insert_batch(&self, entities: Vec<(String, MetadataEntity)>) {
    for (id, entity) in entities {
      self.cache.insert(id, Arc::new(entity)).await;
    }
  }

  /// Remove a metadata entity from cache
  pub async fn invalidate(&self, id: &str) {
    self.cache.invalidate(id).await;
  }

  /// Clear all entries from cache
  pub fn clear(&self) {
    self.cache.invalidate_all();
  }

  /// Get cache statistics (entry count, weighted size)
  pub fn stats(&self) -> (u64, u64) {
    (self.cache.entry_count(), self.cache.weighted_size())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_cache_manager() {
    let cache = CacheManager::new(100);
    let entity = MetadataEntity::new("test-id", "character", "global", serde_json::json!({}), "test");
    cache.insert("test-id".to_string(), entity.clone()).await;
    let cached = cache.get("test-id").await;
    assert!(cached.is_some());
  }
}
