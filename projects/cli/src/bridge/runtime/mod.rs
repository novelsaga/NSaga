//! 运行时管理模块
//!
//! 负责检测和管理 JavaScript 运行时 (Node.js, Bun, Deno)

pub mod discovery;
pub mod process;

pub use discovery::{RuntimeDiscovery, RuntimeInfo, RuntimeType};
pub use process::RuntimeProcess;

#[cfg(test)]
mod tests;
