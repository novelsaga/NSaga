pub mod cache;
/// Metadata system for managing document metadata entities
pub mod index;
pub mod watcher;
pub mod worker;

#[allow(unused_imports)]
pub use cache::CacheManager;  // TODO: integrate into CLI commands
#[allow(unused_imports)]
pub use index::IndexManager;   // TODO: integrate into CLI commands
#[allow(unused_imports)]
pub use watcher::{FileChangeEvent, FileWatcher};  // TODO: integrate into CLI commands
#[allow(unused_imports)]
pub use worker::{WriteBackWorker, WriteTask};  // TODO: integrate into CLI commands
