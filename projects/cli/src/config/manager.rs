use std::{
  collections::{HashMap, LinkedList},
  path::{Path, PathBuf},
  sync::{Arc, LazyLock, RwLock, mpmc::channel},
};

use novelsaga_core::config::{
  CONFIG_FILE_EXTENSIONS_BASE, CONFIG_FILE_EXTENSIONS_EXT_JS, CONFIG_FILE_EXTENSIONS_EXT_TS, CONFIG_FILE_NAMES,
  IGNORE_CONFIG_FILE_NAMES, NovelSagaConfig, OverridableConfig, RootConfig, workspace::WorkspaceConfig,
};

use crate::args::GLOBAL_CLI;

pub static CONFIG_MANAGER: LazyLock<ConfigManager> =
  LazyLock::new(|| ConfigManager::init(GLOBAL_CLI.is_js_supported(), GLOBAL_CLI.is_ts_supported()));

#[derive(Debug)]
struct ScopeMaps {
  pub scope_cache: HashMap<PathBuf, OverridableConfig>,
  pub scope_parents: HashMap<PathBuf, Option<PathBuf>>,
}

#[derive(Debug)]
pub struct ConfigManager {
  // 全局根配置（只加载一次，不可修改）
  root_config: RootConfig,
  scope_maps: Arc<RwLock<ScopeMaps>>,
  js_supported: bool,
  ts_supported: bool,
}

impl ConfigManager {
  /// 初始化配置管理器
  /// 获取root配置
  /// 缓存所有可覆盖配置
  pub fn init(js_supported: bool, ts_supported: bool) -> Self {
    let (root_dir, root_config) = if let Ok(root_config_file) =
      Self::find_root_config_file(&std::env::current_dir().unwrap(), js_supported, ts_supported)
    {
      let root_config = Self::load_root_config_file(&root_config_file, js_supported, ts_supported)
        .expect("Failed to load root config file");
      (root_config_file.parent().unwrap().to_path_buf(), root_config)
    } else {
      // 未找到配置文件，使用默认配置
      (std::env::current_dir().unwrap(), NovelSagaConfig::default())
    };
    let scope_maps: Arc<RwLock<ScopeMaps>> = Arc::new(RwLock::new(ScopeMaps {
      scope_cache: HashMap::new(),
      scope_parents: HashMap::new(),
    }));
    // 从 root_dir 向下预加载所有可覆盖配置文件（多线程）
    let ns_ws_config = if let Some(root_config) = root_config.clone().root {
      root_config.workspace.unwrap_or_default()
    } else {
      WorkspaceConfig::default()
    };
    let novelsaga_dir_path = root_dir.join(ns_ws_config.novelsaga_dir);
    let (tx, _rx) = channel();
    let mut binding = ignore::WalkBuilder::new(&novelsaga_dir_path);
    let parallel_builder = binding
      .git_global(ns_ws_config.respect_git)
      .hidden(false)
      .follow_links(true)
      .max_depth(Some(64))
      .git_global(ns_ws_config.respect_git)
      .require_git(false)
      .git_exclude(true);
    for ignore_file_name in IGNORE_CONFIG_FILE_NAMES {
      parallel_builder.add_ignore(ignore_file_name);
    }
    let parallel = parallel_builder.build_parallel();
    parallel.run(|| {
      let tx = tx.clone();
      Box::new(move |entry: Result<ignore::DirEntry, ignore::Error>| {
        match entry {
          Ok(dirent) => {
            let path_buf = dirent.path().to_path_buf();
            // 处理配置文件
            tx.send(path_buf).ok();
          }
          Err(err) => eprintln!("walk error: {}", err),
        }
        ignore::WalkState::Continue
      })
    });
    drop(tx);

    Self {
      root_config: root_config.root.unwrap_or_default(),
      scope_maps,
      js_supported,
      ts_supported,
    }
  }

  pub fn load_root_config_file(
    path: &Path,
    _js_supported: bool,
    _ts_supported: bool,
  ) -> Result<NovelSagaConfig, config::ConfigError> {
    let config = config::Config::builder().add_source(config::File::from(path)).build()?;
    config.try_deserialize()
  }

  pub fn load_override_config_file(path: &Path) -> Result<OverridableConfig, config::ConfigError> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    if CONFIG_FILE_EXTENSIONS_BASE.contains(&ext) {
      // JS/JSON 等可被 `config` 直接加载的文件
      let cfg = config::Config::builder().add_source(config::File::from(path)).build()?;
      cfg.try_deserialize::<OverridableConfig>()
    } else {
      // md loader --- 自定义解析 frontmatter 返回 OverridableConfig
      let cfg = config::Config::builder()
        .add_source(config::File::new(
          path.to_str().unwrap(),
          super::fileformat::NovelSagaFileFormat::Markdown,
        ))
        .build()?;
      cfg.try_deserialize::<OverridableConfig>()
    }
  }

  /// 输入必须是绝对路径、目录路径
  fn find_root_config_file(start_path: &Path, js_supported: bool, ts_supported: bool) -> Result<PathBuf, ()> {
    let mut current_path = start_path;
    let config_file_extensions = CONFIG_FILE_EXTENSIONS_BASE
      .iter()
      .copied()
      .chain(if js_supported {
        CONFIG_FILE_EXTENSIONS_EXT_JS.iter().copied()
      } else {
        [].iter().copied()
      })
      .chain(if ts_supported {
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
            if let Ok(config) = Self::load_root_config_file(&candidate,js_supported,ts_supported)
              && let Some(root) = config.root
              // 判断workspace字段是否存在，非None即可
              && root.workspace.is_some()
            {
              return Ok(candidate);
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
      return candidates.into_iter().next().ok_or(());
    }
    Err(())
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
    dbg!(&assets_test_dir);
    let result = super::ConfigManager::find_root_config_file(&assets_test_dir, true, true);
    assert!(result.is_ok());
    let config_path = result.unwrap();
    dbg!(&config_path);
  }
  #[test]
  fn test_load_md_config_file() {
    let current_dir = env!("CARGO_MANIFEST_DIR");
    let assets_test_md = std::path::PathBuf::from(current_dir)
      .join("assets")
      .join("test")
      .join("config")
      .join("override_article")
      .join("sub")
      .join("test.md");
    dbg!(&assets_test_md);
    let result = super::ConfigManager::load_override_config_file(&assets_test_md);
    if let Err(e) = &result {
      eprintln!("Error loading config: {:?}", e);
    }
    assert!(result.is_ok());
    let config = result.unwrap();
    dbg!(&config);
    assert!(config.fmt.indent_spaces == 3);
  }
}
