use std::{
  path::{Path, PathBuf},
  sync::{LazyLock, RwLock},
};

use super::schema::{CONFIG_FILE_EXTENSIONS, CONFIG_FILE_NAMES};
use crate::config::NovelSagaConfig;

/// 全局配置加载器实例
/// 自动根据 CLI 参数初始化
pub static GLOBAL_CONFIG_LOADER: LazyLock<RwLock<ConfigLoader>> = LazyLock::new(|| {
  let cli = &*crate::args::GLOBAL_CLI;
  let pwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
  let loader = if let Some(ref config_file) = cli.config {
    ConfigLoader::new_with_file(config_file)
  } else {
    ConfigLoader::new(pwd)
  };

  RwLock::new(loader)
});

/// 配置文件加载器
#[derive(Debug)]
pub struct ConfigLoader {
  /// 当前加载的配置
  config: Option<NovelSagaConfig>,
}

impl ConfigLoader {
  /// 创建新的配置加载器并从指定目录加载配置
  ///
  /// # 参数
  /// - `pwd`: 当前工作目录，将从此目录开始向上搜索配置文件
  pub fn new<P: AsRef<Path>>(pwd: P) -> Self {
    let config = Self::load_from_path(pwd.as_ref());
    Self { config }
  }

  /// 从指定的配置文件路径创建配置加载器
  ///
  /// # 参数
  /// - `config_file`: 配置文件的完整路径
  pub fn new_with_file<P: AsRef<Path>>(config_file: P) -> Self {
    let config_file = config_file.as_ref();
    let config = match Self::load_config_file(config_file) {
      Ok(config) => Some(config),
      Err(e) => {
        eprintln!("Failed to load config file {}: {}", config_file.display(), e);
        None
      }
    };
    Self { config }
  }

  /// 获取当前加载的配置
  pub fn get_config(&self) -> Option<&NovelSagaConfig> {
    self.config.as_ref()
  }

  /// 从指定路径向上搜索最近的配置文件并加载配置
  /// 返回加载的配置对象
  fn load_from_path(start_path: &Path) -> Option<NovelSagaConfig> {
    // 规范化起始路径
    let mut current_path = if start_path.is_file() {
      start_path.parent().unwrap_or(start_path)
    } else {
      start_path
    };

    // 向上搜索直到找到第一个配置文件
    loop {
      // 在当前目录查找配置文件（按字母表顺序）
      if let Some(config_file) = Self::find_config_in_directory(current_path) {
        // 读取并解析配置文件
        match Self::load_config_file(&config_file) {
          Ok(config) => {
            return Some(config);
          }
          Err(e) => {
            eprintln!("Failed to load config file {}: {}", config_file.display(), e);
            return None;
          }
        }
      }

      // 尝试移动到父目录
      match current_path.parent() {
        Some(parent) => current_path = parent,
        None => break, // 已到达根目录
      }
    }

    None
  }

  /// 加载并解析配置文件
  fn load_config_file(path: &Path) -> Result<NovelSagaConfig, config::ConfigError> {
    let config = config::Config::builder().add_source(config::File::from(path)).build()?;
    config.try_deserialize::<NovelSagaConfig>()
  }

  /// 在指定目录中查找第一个配置文件（按字母表顺序）
  fn find_config_in_directory(dir: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    // 收集所有可能的配置文件路径
    for file_name in CONFIG_FILE_NAMES {
      for extension in CONFIG_FILE_EXTENSIONS {
        let config_path = dir.join(format!("{file_name}.{extension}"));
        if config_path.exists() && config_path.is_file() {
          candidates.push(config_path);
        }
      }
    }
    // 按字母表顺序排序
    candidates.sort();
    // 返回第一个（字母表最前的）
    candidates.into_iter().next()
  }
}
