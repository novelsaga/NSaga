//! Asset 路径定位器

#[cfg(not(debug_assertions))]
use std::fs;
use std::path::PathBuf;

#[cfg(not(debug_assertions))]
use directories::ProjectDirs;

#[cfg(not(debug_assertions))]
use super::EmbeddedAssets;
use super::{
  ENV_ASSET_DIR,
  error::{AssetError, Result},
};

/// Asset 资源定位器
///
/// 提供统一的资源文件查找接口
pub struct AssetLocator;

impl AssetLocator {
  /// 查找指定的 asset 文件
  ///
  /// # Arguments
  ///
  /// * `asset_name` - 资源文件名（如 "config-bridge.js"）
  ///
  /// # 优先级
  ///
  /// 1. 环境变量 `NSAGA_CLI_ASSET_DIR`
  /// 2. 开发环境（debug build）：`CARGO_MANIFEST_DIR` 相对路径
  /// 3. 生产环境（release build）：编译期指定路径
  ///
  /// # Errors
  ///
  /// 当无法找到资源文件时返回 `BridgeError::ResourceNotFound`
  pub fn locate(asset_name: &str) -> Result<PathBuf> {
    // 优先级 1: 环境变量
    if let Ok(dir) = std::env::var(ENV_ASSET_DIR) {
      let path = PathBuf::from(dir).join(asset_name);
      if path.exists() {
        return Ok(path);
      }
    }

    // 优先级 2/3: 条件编译
    #[cfg(debug_assertions)]
    {
      Self::locate_dev(asset_name)
    }

    #[cfg(not(debug_assertions))]
    {
      Self::locate_prod(asset_name)
    }
  }

  /// 开发环境路径查找
  #[cfg(debug_assertions)]
  fn locate_dev(asset_name: &str) -> Result<PathBuf> {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
      let path = PathBuf::from(manifest_dir)
        .ancestors()
        .nth(2)
        .unwrap()
        .join("projects/cli/assets")
        .join(asset_name);
      if path.exists() {
        return Ok(path);
      }
    }

    Err(AssetError::NotFound(
      asset_name.to_string(),
      format!("Set {ENV_ASSET_DIR} or run from workspace root."),
    ))
  }

  /// 生产环境路径查找 (Embedded)
  #[cfg(not(debug_assertions))]
  fn locate_prod(asset_name: &str) -> Result<PathBuf> {
    // 1. 确定缓存目录: ~/.cache/novelsaga/assets/<version>
    let dirs = ProjectDirs::from("rs", "novelsaga", "novelsaga").ok_or_else(|| {
      AssetError::IoError(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Could not determine home directory",
      ))
    })?;

    let version = env!("CARGO_PKG_VERSION");
    let cache_dir = dirs.cache_dir().join("assets").join(version);

    // 如果缓存目录不存在，首先尝试全量解压所有 assets
    // 这确保 package.json 等依赖文件也被正确释放
    if !cache_dir.exists() {
      for file in EmbeddedAssets::iter() {
        let file_path = file.as_ref();
        if let Some(embedded_file) = EmbeddedAssets::get(file_path) {
          let target = cache_dir.join(file_path);
          if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(AssetError::IoError)?;
          }
          fs::write(&target, embedded_file.data).map_err(AssetError::IoError)?;
        }
      }
    }

    let target_path = cache_dir.join(asset_name);

    // 2. 如果文件已存在，直接返回
    if target_path.exists() {
      return Ok(target_path);
    }

    // 3. Fallback: 如果全量解压后文件仍不存在（或目录已存在但文件确实），尝试单独查找并释放
    // asset_name 必须匹配 embedded folder 的相对路径
    // asset_name 必须匹配 embedded folder 的相对路径
    // EmbeddedAssets folder="assets", 所以 "js/dist/xxx" 对应 assets/js/dist/xxx
    if let Some(embedded_file) = EmbeddedAssets::get(asset_name) {
      if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent).map_err(AssetError::IoError)?;
      }
      fs::write(&target_path, embedded_file.data).map_err(AssetError::IoError)?;
      Ok(target_path)
    } else {
      Err(AssetError::NotFound(
        asset_name.to_string(),
        "Embedded asset not found".into(),
      ))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[ignore = "requires built assets"]
  fn test_locate_config_bridge() {
    // 开发环境应该能找到 config-bridge.js
    let result = AssetLocator::locate("js/dist/config-bridge.js");
    assert!(result.is_ok(), "Should find config-bridge.js in dev environment");
  }

  #[test]
  fn test_env_override() {
    use std::fs;

    use tempfile::tempdir;

    let dir = tempdir().expect("Failed to create temp dir");
    let test_file = dir.path().join("test.js");
    fs::write(&test_file, "// test").expect("Failed to write test file");

    unsafe {
      std::env::set_var(super::ENV_ASSET_DIR, dir.path());
    }

    let result = AssetLocator::locate("test.js");
    assert!(result.is_ok(), "Should find file via env var");
    assert_eq!(result.unwrap(), test_file);

    unsafe {
      std::env::remove_var(super::ENV_ASSET_DIR);
    }
  }
}
