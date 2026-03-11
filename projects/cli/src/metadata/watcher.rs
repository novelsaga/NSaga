//! **DEPRECATED**: FileWatcher is no longer actively maintained.
//!
//! This module has been superseded by LSP's `didChangeWatchedFiles` protocol,
//! which provides better integration with editor file watching mechanisms.
//!
//! Kept for reference and potential future use if direct filesystem watching
//! becomes necessary again. New code should NOT depend on this module.
//!
use std::{
  path::Path,
  sync::mpsc::{Receiver, Sender, channel},
};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// File change events filtered to metadata files
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FileChangeEvent {
  Created(String),
  Modified(String),
  Removed(String),
}

/// Watches metadata directory for file changes
#[allow(dead_code)]
pub struct FileWatcher {
  watcher: RecommendedWatcher,
  receiver: Receiver<FileChangeEvent>,
}

#[allow(dead_code)]
impl FileWatcher {
  /// Create a new `FileWatcher` for the given path
  pub fn new(_watch_path: &Path) -> Result<Self, notify::Error> {
    let (sender, receiver) = channel::<FileChangeEvent>();

    let watcher = RecommendedWatcher::new(
      move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
          Self::handle_event(&event, &sender);
        }
      },
      Config::default(),
    )?;

    Ok(FileWatcher { watcher, receiver })
  }

  /// Handle a file system event and send filtered events
  fn handle_event(event: &Event, sender: &Sender<FileChangeEvent>) {
    for path in &event.paths {
      let path_str = path.to_string_lossy();

      // Filter: only /metadata/ paths ending with .md
      if !path_str.contains("/metadata/") || !path_str.ends_with(".md") {
        continue;
      }

      let file_name = path_str.to_string();

      let change_event = match &event.kind {
        EventKind::Create(_) => FileChangeEvent::Created(file_name),
        EventKind::Modify(_) => FileChangeEvent::Modified(file_name),
        EventKind::Remove(_) => FileChangeEvent::Removed(file_name),
        _ => continue,
      };

      if let Err(e) = sender.send(change_event) {
        eprintln!("Failed to send file change event: {e}");
      }
    }
  }

  /// Watch a path recursively
  pub fn watch(&mut self, path: &Path) -> Result<(), notify::Error> {
    self.watcher.watch(path, RecursiveMode::Recursive)
  }

  /// Stop watching a path
  pub fn unwatch(&mut self, path: &Path) -> Result<(), notify::Error> {
    self.watcher.unwatch(path)
  }

  /// Try to receive a file change event (non-blocking)
  pub fn try_recv(&self) -> Option<FileChangeEvent> {
    self.receiver.try_recv().ok()
  }

  /// Receive a file change event (blocking)
  pub fn recv(&self) -> Result<FileChangeEvent, std::sync::mpsc::RecvError> {
    self.receiver.recv()
  }
}

#[cfg(test)]
mod tests {
  use std::{fs, thread, time::Duration};

  use tempfile::TempDir;

  use super::*;

  #[test]
  fn test_file_watcher() {
    // Create a temporary directory with metadata subdirectory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let metadata_dir = temp_dir.path().join("metadata");
    fs::create_dir(&metadata_dir).expect("Failed to create metadata dir");

    // Create the watcher
    let mut watcher = FileWatcher::new(temp_dir.path()).expect("Failed to create watcher");
    watcher.watch(temp_dir.path()).expect("Failed to watch directory");

    // Give the watcher a moment to initialize
    thread::sleep(Duration::from_millis(100));

    // Test 1: Create a markdown file in metadata dir
    let test_file = metadata_dir.join("test.md");
    fs::write(&test_file, "# Test").expect("Failed to write file");

    thread::sleep(Duration::from_millis(200));

    // Should receive Created event
    let mut received_create = false;
    for _ in 0..5 {
      if matches!(watcher.try_recv(), Some(FileChangeEvent::Created(_))) {
        received_create = true;
        break;
      }
      thread::sleep(Duration::from_millis(50));
    }
    assert!(received_create, "Should receive Created event");

    // Test 2: Modify the file
    fs::write(&test_file, "# Test Modified").expect("Failed to modify file");

    thread::sleep(Duration::from_millis(200));

    let mut received_modify = false;
    for _ in 0..5 {
      if matches!(watcher.try_recv(), Some(FileChangeEvent::Modified(_))) {
        received_modify = true;
        break;
      }
      thread::sleep(Duration::from_millis(50));
    }
    assert!(received_modify, "Should receive Modified event");

    // Test 3: Remove the file
    fs::remove_file(&test_file).expect("Failed to remove file");

    thread::sleep(Duration::from_millis(200));

    let mut received_remove = false;
    for _ in 0..5 {
      if matches!(watcher.try_recv(), Some(FileChangeEvent::Removed(_))) {
        received_remove = true;
        break;
      }
      thread::sleep(Duration::from_millis(50));
    }
    assert!(received_remove, "Should receive Removed event");

    // Test 4: Files not in metadata dir should be ignored
    let other_file = temp_dir.path().join("other.md");
    fs::write(&other_file, "# Other").expect("Failed to write other file");

    thread::sleep(Duration::from_millis(200));

    // Should not receive any event (not in metadata/)
    let should_be_none = watcher.try_recv();
    assert!(
      should_be_none.is_none(),
      "Should ignore files not in metadata directory"
    );

    // Test 5: Non-markdown files in metadata should be ignored
    let txt_file = metadata_dir.join("test.txt");
    fs::write(&txt_file, "Test").expect("Failed to write txt file");

    thread::sleep(Duration::from_millis(200));

    // Should not receive any event (not .md)
    let should_be_none = watcher.try_recv();
    assert!(should_be_none.is_none(), "Should ignore non-markdown files");
  }
}
