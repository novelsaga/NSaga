//! Bridge 资源文件管理模块
//!
//! 负责定位 Bridge 所需的 JavaScript 资源文件。
//!
//! # 路径查找策略
//!
//! 1. **环境变量** `NSAGA_CLI_ASSET_DIR` - 最高优先级，用于运行时覆盖
//! 2. **开发环境** (debug build) - 基于 `CARGO_MANIFEST_DIR` 的相对路径
//! 3. **生产环境** (release build) - 内嵌资源 (Embedded Assets)，自动释放到用户缓存目录

mod error;
mod locator;

#[allow(unused_imports)]
pub use error::{AssetError, Result};
pub use locator::AssetLocator;
// Re-export rust-embed for internal usage if needed
#[cfg(not(debug_assertions))]
use rust_embed::RustEmbed;

/// 环境变量：资源目录路径
pub const ENV_ASSET_DIR: &str = "NSAGA_CLI_ASSET_DIR";

/// 内嵌资源定义
#[cfg(not(debug_assertions))]
#[derive(RustEmbed)]
#[folder = "assets"]
pub struct EmbeddedAssets;
