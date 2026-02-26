use std::{sync::Arc, time::Duration};

use flume::{Receiver, Sender};
use novelsaga_core::metadata::model::MetadataEntity;
use tokio::time::interval;

use crate::metadata::index::IndexManager;

/// Represents a write task to be processed by the worker
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum WriteTask {
  /// Upsert (insert or update) a metadata entity
  Upsert { id: String, data: Vec<u8> },
  /// Delete a metadata entity by ID
  Delete { id: String },
  /// Flush all pending writes to persistent storage
  Flush,
}

/// `WriteBackWorker` processes metadata write tasks in batches
#[allow(dead_code)]
pub struct WriteBackWorker {
  task_sender: Sender<WriteTask>,
  worker_handle: tokio::task::JoinHandle<()>,
}

#[allow(dead_code)]
impl WriteBackWorker {
  /// Creates a new `WriteBackWorker` and starts the background worker task
  ///
  /// # Arguments
  ///
  /// * `index` - Shared `IndexManager` for persisting metadata
  /// * `batch_size` - Maximum number of tasks to batch before processing
  /// * `flush_interval` - Time interval for flushing partial batches
  ///
  /// # Returns
  ///
  /// A new `WriteBackWorker` instance with a running background task
  pub fn new(index: Arc<IndexManager>, batch_size: usize, flush_interval: Duration) -> Self {
    let (task_sender, task_receiver) = flume::unbounded();

    let worker_handle = tokio::spawn(Self::run_worker(task_receiver, index, batch_size, flush_interval));

    WriteBackWorker {
      task_sender,
      worker_handle,
    }
  }

  /// Submits a write task to the worker queue
  ///
  /// # Arguments
  ///
  /// * `task` - The `WriteTask` to be processed
  pub fn submit(&self, task: WriteTask) {
    if let Err(e) = self.task_sender.send(task) {
      eprintln!("Failed to submit task to WriteBackWorker: {e}");
    }
  }

  /// Background worker task that processes tasks in batches
  ///
  /// Collects tasks and processes them either when the batch is full or when the flush timer fires.
  /// Uses `tokio::select`! to handle both conditions.
  async fn run_worker(
    receiver: Receiver<WriteTask>,
    index: Arc<IndexManager>,
    batch_size: usize,
    flush_interval: Duration,
  ) {
    let mut batch = Vec::with_capacity(batch_size);
    let mut ticker = interval(flush_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
      tokio::select! {
          // Receive tasks from the channel
          result = receiver.recv_async() => {
              if let Ok(task) = result {
                  batch.push(task);

                  // Process batch if it's full
                  if batch.len() >= batch_size {
                      Self::process_batch(&batch, &index).await;
                      batch.clear();
                      ticker.reset();
                  }
              } else {
                  // Channel closed, process remaining tasks and exit
                  if !batch.is_empty() {
                      Self::process_batch(&batch, &index).await;
                  }
                  break;
              }
          }

          // Flush batch on timer
          _ = ticker.tick() => {
              if !batch.is_empty() {
                  Self::process_batch(&batch, &index).await;
                  batch.clear();
              }
          }
      }
    }
  }

  /// Processes a batch of write tasks
  ///
  /// # Arguments
  ///
  /// * `batch` - Slice of tasks to process
  /// * `index` - Reference to the `IndexManager`
  #[allow(clippy::unused_async)]
  async fn process_batch(batch: &[WriteTask], index: &IndexManager) {
    for task in batch {
      match task {
        WriteTask::Upsert { id, data } => {
          // Deserialize the data into a MetadataEntity
          match serde_json::from_slice::<MetadataEntity>(data) {
            Ok(entity) => {
              if let Err(e) = index.index_entity(&entity) {
                eprintln!("Failed to index entity {id}: {e}");
              }
            }
            Err(e) => {
              eprintln!("Failed to deserialize entity {id}: {e}");
            }
          }
        }
        WriteTask::Delete { id } => {
          if let Err(e) = index.remove_entity(id) {
            eprintln!("Failed to remove entity {id}: {e}");
          }
        }
        WriteTask::Flush => {
          if let Err(e) = index.flush() {
            eprintln!("Failed to flush index: {e}");
          }
        }
      }
    }

    // Flush after processing batch
    if let Err(e) = index.flush() {
      eprintln!("Failed to flush index after batch: {e}");
    }
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;
  use tempfile::TempDir;

  use super::*;

  #[tokio::test]
  async fn test_worker_upsert_batch() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let index = Arc::new(IndexManager::open(temp_dir.path()).expect("Failed to open index"));

    let worker = WriteBackWorker::new(index.clone(), 2, Duration::from_secs(1));

    // Create test entity data
    let entity1 = MetadataEntity::new("test-id-1", "article", "blog", json!({"title": "Test 1"}), "Body 1");
    let entity1_data = serde_json::to_vec(&entity1).expect("Failed to serialize entity");

    let entity2 = MetadataEntity::new("test-id-2", "article", "blog", json!({"title": "Test 2"}), "Body 2");
    let entity2_data = serde_json::to_vec(&entity2).expect("Failed to serialize entity");

    // Submit upsert tasks
    worker.submit(WriteTask::Upsert {
      id: "test-id-1".to_string(),
      data: entity1_data,
    });
    worker.submit(WriteTask::Upsert {
      id: "test-id-2".to_string(),
      data: entity2_data,
    });

    // Give worker time to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify entities were indexed
    let retrieved1 = index.get_by_id("test-id-1").expect("Failed to query index");
    assert!(retrieved1.is_some());
    assert_eq!(retrieved1.as_ref().unwrap().id, "test-id-1");

    let retrieved2 = index.get_by_id("test-id-2").expect("Failed to query index");
    assert!(retrieved2.is_some());
    assert_eq!(retrieved2.as_ref().unwrap().id, "test-id-2");
  }

  #[tokio::test]
  async fn test_worker_delete() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let index = Arc::new(IndexManager::open(temp_dir.path()).expect("Failed to open index"));

    // Pre-populate the index
    let entity = MetadataEntity::new("test-id", "article", "blog", json!({"title": "To Delete"}), "Body");
    index.index_entity(&entity).expect("Failed to index entity");

    let worker = WriteBackWorker::new(index.clone(), 2, Duration::from_secs(1));

    // Submit delete task
    worker.submit(WriteTask::Delete {
      id: "test-id".to_string(),
    });

    // Give worker time to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify entity was deleted
    let retrieved = index.get_by_id("test-id").expect("Failed to query index");
    assert!(retrieved.is_none());
  }

  #[tokio::test]
  async fn test_worker_flush_on_timeout() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let index = Arc::new(IndexManager::open(temp_dir.path()).expect("Failed to open index"));

    // Create worker with small batch size and short flush interval
    let worker = WriteBackWorker::new(index.clone(), 10, Duration::from_millis(100));

    // Create test entity data
    let entity = MetadataEntity::new("test-id", "article", "blog", json!({"title": "Test"}), "Body");
    let entity_data = serde_json::to_vec(&entity).expect("Failed to serialize entity");

    // Submit single task (less than batch_size)
    worker.submit(WriteTask::Upsert {
      id: "test-id".to_string(),
      data: entity_data,
    });

    // Wait for flush interval to pass
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Verify entity was flushed to persistent storage
    let retrieved = index.get_by_id("test-id").expect("Failed to query index");
    assert!(retrieved.is_some());
  }

  #[tokio::test]
  async fn test_worker_mixed_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let index = Arc::new(IndexManager::open(temp_dir.path()).expect("Failed to open index"));

    let worker = WriteBackWorker::new(index.clone(), 3, Duration::from_secs(1));

    // Pre-populate an entity to delete
    let entity_to_delete = MetadataEntity::new("delete-me", "article", "blog", json!({}), "To be deleted");
    index.index_entity(&entity_to_delete).expect("Failed to index entity");

    // Create new entity to upsert
    let entity_new = MetadataEntity::new("new-id", "comment", "blog", json!({"author": "test"}), "New body");
    let entity_new_data = serde_json::to_vec(&entity_new).expect("Failed to serialize");

    // Submit mixed tasks
    worker.submit(WriteTask::Upsert {
      id: "new-id".to_string(),
      data: entity_new_data,
    });
    worker.submit(WriteTask::Delete {
      id: "delete-me".to_string(),
    });
    worker.submit(WriteTask::Flush);

    // Give worker time to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Verify operations
    let new_entity = index.get_by_id("new-id").expect("Failed to query");
    assert!(new_entity.is_some());

    let deleted_entity = index.get_by_id("delete-me").expect("Failed to query");
    assert!(deleted_entity.is_none());
  }
}
