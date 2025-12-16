use std::{
  collections::{HashMap, LinkedList},
  path::PathBuf,
  sync::LazyLock,
};

use novelsaga_core::config::{
  CONFIG_FILE_EXTENSIONS_BASE, CONFIG_FILE_EXTENSIONS_EXT_JS, CONFIG_FILE_EXTENSIONS_EXT_TS, CONFIG_FILE_NAMES, Config,
  NovelSagaConfig, OverridableConfig, RootConfig,
};

use crate::args::GLOBAL_CLI;

pub static CONFIG_MANAGER: LazyLock<ConfigManager> =
  LazyLock::new(|| ConfigManager::new(GLOBAL_CLI.is_js_supported(), GLOBAL_CLI.is_ts_supported()));

#[derive(Debug)]
pub struct ConfigManager {
  // 全局根配置（只加载一次，不可修改）
  root_config: RootConfig,

  // 每个 scope 的可覆盖配置缓存
  scope_cache: HashMap<PathBuf, OverridableConfig>,

  // scope 的继承链（用于组合）
  scope_parents: HashMap<PathBuf, Option<PathBuf>>,

  js_supported: bool,
  ts_supported: bool,
}

impl ConfigManager {
  fn new(js_supported: bool, ts_supported: bool) -> Self {
    let root_config = RootConfig::default();
    let scope_cache = HashMap::new();
    let scope_parents = HashMap::new();
    Self {
      root_config,
      scope_cache,
      scope_parents,
      js_supported,
      ts_supported,
    }
  }

  /// `load_config_file::<NovelSagaConfig>` 获取root配置文件
  /// `load_config_file::<OverridableConfig>` 获取可覆盖配置文件
  fn load_config_file<T: Config + serde::Deserialize<'static>>(path: &PathBuf) -> Result<T, config::ConfigError> {
    let config = config::Config::builder()
      .add_source(config::File::from(path.as_path()))
      .build()?;
    config.try_deserialize::<T>()
  }

  /// 输入必须是绝对路径、目录路径
  fn find_root_config_file(&self, start_path: &PathBuf) -> Option<PathBuf> {
    let mut current_path = start_path.as_path();
    let config_file_extensions = CONFIG_FILE_EXTENSIONS_BASE
      .iter()
      .copied()
      .chain(if self.js_supported {
        CONFIG_FILE_EXTENSIONS_EXT_JS.iter().copied()
      } else {
        [].iter().copied()
      })
      .chain(if self.ts_supported {
        CONFIG_FILE_EXTENSIONS_EXT_TS.iter().copied()
      } else {
        [].iter().copied()
      });
    let mut searched_paths: LinkedList<PathBuf> = LinkedList::new();
    loop {
      // 从当前目录一直向上查询，发现配置文件则进行解析查看是否存在 root.workspace 字段，若一个workspace字段都没有则返回最顶层配置文件
      // 相同层级若存在有ext不同的多个配置文件，则按字母表顺序优先加载
      for file_name in CONFIG_FILE_NAMES {
        for extension in config_file_extensions.clone() {
          let candidate = current_path.join(format!("{file_name}.{extension}"));
          searched_paths.push_back(candidate.clone());
          if candidate.try_exists().unwrap_or(false) {
            // 尝试加载配置文件
            if let Ok(config) = Self::load_config_file::<NovelSagaConfig>(&candidate)
              && let Some(root) = config.root
              // 判断workspace字段是否存在，非None即可
              && root.workspace.is_some()
            {
              return Some(candidate);
            }
          }
        }
      }
      // 一直访问到没有读权限为止
      match current_path.parent() {
        Some(parent) => current_path = parent,
        None => break, // 已到达根目录
      }
    }
    dbg!("No workspace config found, searching for top-level config file...");
    // 未找到包含 workspace 字段的配置文件，返回最顶层找到的配置文件（若有）（从back向前找）
    let root_file = searched_paths
      .into_iter()
      .rev()
      .find(|path| path.try_exists().unwrap_or(false));
    // 寻找其同级别下字母表最前的配置文件
    if let Some(root_file) = root_file {
      let parent_dir = root_file.parent().unwrap();
      let mut candidates: Vec<PathBuf> = Vec::new();
      for file_name in CONFIG_FILE_NAMES {
        for extension in config_file_extensions.clone() {
          let candidate = parent_dir.join(format!("{file_name}.{extension}"));
          if candidate.try_exists().unwrap_or(false) {
            candidates.push(candidate);
          }
        }
      }
      candidates.sort();
      return candidates.into_iter().next();
    }
    None
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn test_find_root_config_file() {
    // 获取当前开发目录路径
    let current_dir = env!("CARGO_MANIFEST_DIR");
    let assets_test_dir = std::path::PathBuf::from(current_dir)
      .join("assets")
      .join("test")
      .join("config")
      .join("override_base")
      .join("sub");
    dbg!("Testing find_root_config_file in directory:", &assets_test_dir);
    let config_manager = super::ConfigManager::new(true, true);
    let found_config = config_manager.find_root_config_file(&assets_test_dir);
    assert!(found_config.is_some());
    let found_config = found_config.unwrap();
    println!("Found config file: {}", found_config.display());
  }
}
