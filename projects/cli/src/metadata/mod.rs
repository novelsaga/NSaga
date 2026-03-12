pub mod cache;
/// Metadata system for managing document metadata entities
/// Metadata system for managing document metadata entities
pub mod index;
pub mod resolver;
#[allow(unused_imports)]
pub use resolver::{MetadataResolver, ResolutionContext, ResolverError}; // TODO: integrate into CLI commands
// pub mod watcher; // DEPRECATED: FileWatcher is superseded by LSP didChangeWatchedFiles in P3
pub mod worker;

#[allow(unused_imports)]
pub use cache::CacheManager; // TODO: integrate into CLI commands
pub use index::IndexManager; // TODO: integrate into CLI commands
// pub use watcher::{FileChangeEvent, FileWatcher}; // DEPRECATED: Use LSP watched-files protocol instead
#[allow(unused_imports)]
pub use worker::{WriteBackWorker, WriteTask}; // TODO: integrate into CLI commands
