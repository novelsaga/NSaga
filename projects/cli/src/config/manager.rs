use std::{
  collections::{HashMap, VecDeque},
  path::{Path, PathBuf},
  sync::{Arc, LazyLock, RwLock},
};

use config::FileStoredFormat;
use novelsaga_core::config::{
  CONFIG_FILE_EXTENSIONS_BASE, CONFIG_FILE_EXTENSIONS_EXT_JS, CONFIG_FILE_EXTENSIONS_EXT_TS, CONFIG_FILE_NAMES,
  IGNORE_CONFIG_FILE_NAMES, NovelSagaConfig, OverridableConfig, RootConfig,
};

use crate::args::GLOBAL_CLI;

pub static CONFIG_MANAGER: LazyLock<Arc<RwLock<ConfigManager>>> = LazyLock::new(|| {
  Arc::new(RwLock::new(ConfigManager::new(
    GLOBAL_CLI.is_js_supported(),
    GLOBAL_CLI.is_ts_supported(),
  )))
});

#[derive(Debug)]
pub struct ConfigManager {
  // 全局根配置（只加载一次，不可修改）
  root_config: RootConfig,
  root_dir: PathBuf,
  cache: Arc<RwLock<HashMap<PathBuf, OverridableConfig>>>,
  js_supported: bool,
  ts_supported: bool,
}

impl ConfigManager {
  /// 初始化配置管理器
  /// 获取root配置
  /// 缓存所有可覆盖配置
  pub fn new(js_supported: bool, ts_supported: bool) -> Self {
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
    let cache: Arc<RwLock<HashMap<PathBuf, OverridableConfig>>> = Arc::new(RwLock::new(HashMap::new()));

    Self {
      root_config: root_config.root.unwrap_or_default(),
      cache,
      root_dir,
      js_supported,
      ts_supported,
    }
  }

  #[allow(dead_code)]
  pub fn get_root_config(&self) -> &RootConfig {
    &self.root_config
  }

  #[allow(dead_code)]
  pub fn get_override_config(&self, path: &Path) -> Result<OverridableConfig, config::ConfigError> {
    // 判断缓存中是否存在（只读锁）
    {
      let cache_read = self.cache.read().unwrap();
      if let Some(cfg) = cache_read.get(path) {
        return Ok(cfg.clone());
      }
    }

    // 加载配置文件
    let cfg = self.load_override_config_file(path)?;
    // 写入缓存（写锁）
    let mut cache_write = self.cache.write().unwrap();
    cache_write.insert(path.to_path_buf(), cfg.clone());
    Ok(cfg)
  }

  #[allow(dead_code)]
  fn get_config_files_on_every_parent_dirs(&self, start_path: &Path) -> VecDeque<PathBuf> {
    let mut current_path = start_path;
    let mut config_files: VecDeque<PathBuf> = VecDeque::new();

    loop {
      // 尝试在当前目录查找配置文件
      if let Ok(candidate) = Self::find_config_file_in_directory(current_path, self.js_supported, self.ts_supported) {
        config_files.push_front(candidate);
      }

      // 向上继续查找，直到 self.root_dir
      if current_path == self.root_dir {
        break;
      }
      match current_path.parent() {
        Some(parent) => current_path = parent,
        None => break, // 已到达根目录
      }
    }

    config_files
  }

  /// 判断一个配置文件是否被 ignore（支持自定义 ignore 文件名，例如 `.novelsagaignore`）
  fn is_ignored_config_file(&self, path: &Path) -> bool {
    // time
    let start_time = std::time::Instant::now();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut binding = ignore::WalkBuilder::new(&self.root_dir);
    let parallel_walker_builder = binding
      .git_global(false)
      .hidden(false)
      .follow_links(true)
      .max_depth(Some(64))
      .git_global(false)
      .require_git(false)
      .git_exclude(false)
      .git_ignore(false)
      .require_git(false)
      .ignore_case_insensitive(
        // windows
        #[cfg(target_os = "windows")]
        true,
        // other os
        #[cfg(not(target_os = "windows"))]
        false,
      );
    for ignore_file_name in IGNORE_CONFIG_FILE_NAMES {
      parallel_walker_builder.add_custom_ignore_filename(ignore_file_name);
    }
    let walker = parallel_walker_builder.build_parallel();
    walker.run(|| {
      let tx = tx.clone();
      Box::new(move |result| {
        if let Ok(entry) = result {
          let entry_path = entry.path();
          if entry_path == path {
            tx.send(true).unwrap();
            return ignore::WalkState::Quit;
          }
        }
        ignore::WalkState::Continue
      })
    });
    let elapsed = start_time.elapsed();
    dbg!(elapsed);
    // walker returns an entry when the path is visible; invert result so that
    // true means "ignored" and false means "not ignored"
    !rx.try_recv().unwrap_or(false)
  }

  fn load_root_config_file(
    path: &Path,
    _js_supported: bool,
    _ts_supported: bool,
  ) -> Result<NovelSagaConfig, config::ConfigError> {
    let config = config::Config::builder().add_source(config::File::from(path)).build()?;
    config.try_deserialize()
  }

  fn load_override_config_file(&self, path: &Path) -> Result<OverridableConfig, config::ConfigError> {
    if self.is_ignored_config_file(path) {
      return Err(config::ConfigError::Message(format!(
        "Ignored config file: {}",
        path.display()
      )));
    }
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    // 加载所有父级目录的配置文件
    let mut cfg_builder_parents = config::Config::builder();
    for parent_cfg_path in self.get_config_files_on_every_parent_dirs(path.parent().unwrap()) {
      let config_file_result =
        Self::find_config_file_in_directory(parent_cfg_path.parent().unwrap(), self.js_supported, self.ts_supported);
      if let Ok(cfg_path) = config_file_result {
        let ext = cfg_path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if CONFIG_FILE_EXTENSIONS_BASE.contains(&ext) {
          // JSON 等可被 `config` 直接加载的文件
          dbg!(cfg_path.display());
          cfg_builder_parents = cfg_builder_parents.add_source(config::File::from(cfg_path));
        }
      }
    }
    dbg!(path.display());
    let cfg_builder_result = if CONFIG_FILE_EXTENSIONS_BASE.contains(&ext) {
      // JSON 等可被 `config` 直接加载的文件
      Ok(cfg_builder_parents.add_source(config::File::from(path)))
    } else if super::fileformat::NovelSagaFileFormat::Markdown
      .file_extensions()
      .contains(&ext)
    {
      // md loader --- 自定义解析 frontmatter 返回 OverridableConfig
      Ok(cfg_builder_parents.add_source(config::File::new(
        path.to_str().unwrap(),
        super::fileformat::NovelSagaFileFormat::Markdown,
      )))
    } else {
      Err(config::ConfigError::Message(format!(
        "Unsupported config file extension: {ext}"
      )))
    };

    match cfg_builder_result {
      Ok(builder) => {
        let cfg = builder.build()?;
        cfg.try_deserialize::<OverridableConfig>()
      }
      Err(e) => Err(e),
    }
  }

  #[allow(dead_code)]
  fn load_override_config_dir(path: &Path) -> Result<OverridableConfig, config::ConfigError> {
    let config = config::Config::builder().add_source(config::File::from(path)).build()?;
    config.try_deserialize::<OverridableConfig>()
  }

  fn find_config_file_in_directory(
    dir: &Path,
    js_supported: bool,
    ts_supported: bool,
  ) -> Result<PathBuf, std::io::Error> {
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
    let mut candidates: Vec<PathBuf> = Vec::new();
    for file_name in CONFIG_FILE_NAMES {
      for extension in config_file_extensions.clone() {
        let candidate = dir.join(format!("{file_name}.{extension}"));
        if candidate.try_exists().unwrap_or(false) {
          candidates.push(candidate);
        }
      }
    }
    candidates.sort();
    candidates.into_iter().next().ok_or(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "Config file not found",
    ))
  }

  /// 输入必须是绝对路径、目录路径
  fn find_root_config_file(
    start_path: &Path,
    js_supported: bool,
    ts_supported: bool,
  ) -> Result<PathBuf, std::io::Error> {
    let mut current_path = start_path;
    let mut searched_paths: VecDeque<PathBuf> = VecDeque::new();

    loop {
      // 尝试在当前目录复用 `find_config_file_in_directory`
      if let Ok(candidate) = Self::find_config_file_in_directory(current_path, js_supported, ts_supported) {
        // 记录已有的配置文件（用于在未找到 workspace 字段时选最顶层）
        searched_paths.push_back(candidate.clone());
        // 尝试加载并检查 workspace 字段
        if let Ok(config) = Self::load_root_config_file(&candidate, js_supported, ts_supported)
          && let Some(root) = config.root
          && root.workspace.is_some()
        {
          return Ok(candidate);
        }
      } else {
        // 当前目录未找到配置文件，继续向上搜索
      }

      // 向上继续查找，直到根目录
      match current_path.parent() {
        Some(parent) => current_path = parent,
        None => break, // 已到达根目录
      }
    }

    dbg!("No workspace config found, searching for top-level config file...");
    // 未找到包含 workspace 字段的配置文件，返回最顶层找到的配置文件（若有）
    let root_file = searched_paths
      .into_iter()
      .rev()
      .find(|path| path.try_exists().unwrap_or(false));
    if let Some(root_file) = root_file {
      let parent_dir = root_file.parent().unwrap();
      // 在其同级目录中复用 `find_config_file_in_directory`，返回字母表最前的配置文件
      return Self::find_config_file_in_directory(parent_dir, js_supported, ts_supported)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::NotFound, "No top-level config file found"));
    }

    Err(std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "No config file found in hierarchy",
    ))
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
      .join("config_manager")
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
    let assets_test_md_dir = std::path::PathBuf::from(current_dir)
      .join("assets")
      .join("test")
      .join("config")
      .join("config_manager")
      .join("sub");
    let assets_test_md = assets_test_md_dir.clone().join("test.md");
    dbg!(&assets_test_md);
    let manager = super::ConfigManager::new(true, true);
    let result = manager.load_override_config_file(&assets_test_md);
    if let Err(e) = &result {
      eprintln!("Error loading config: {e}");
    }
    assert!(result.is_ok());
    let config = result.unwrap();
    dbg!(&config);
    assert!(config.fmt.indent_spaces == 3);
    let assets_test_md_no_matter = assets_test_md_dir.join("test-no-matter.markdown");
    let result_no_matter = manager.load_override_config_file(&assets_test_md_no_matter);
    if let Err(e) = &result_no_matter {
      eprintln!("Error loading config without matter: {e}");
    }
    let config_no_matter = result_no_matter.unwrap();
    dbg!(&config_no_matter);
    assert!(config_no_matter.fmt.indent_spaces == 1);
    let sub_sub_dir = assets_test_md_dir.join("sub").join("novelsaga.config.json");
    let result_sub_sub = manager.load_override_config_file(&sub_sub_dir);
    if let Err(e) = &result_sub_sub {
      eprintln!("Error loading sub/sub config: {e}");
    }
    dbg!(&result_sub_sub);
    assert!(result_sub_sub.is_ok());
    let config_sub_sub = result_sub_sub.unwrap();
    assert!(config_sub_sub.fmt.indent_spaces == 5);
  }

  #[test]
  fn test_is_ignored_config_file() {
    let current_dir = env!("CARGO_MANIFEST_DIR");
    let assets_test_ignore_dir = std::path::PathBuf::from(current_dir)
      .join("assets")
      .join("test")
      .join("config")
      .join("config_manager")
      .join("sub")
      .join("sub")
      .join("test-ignore.md");
    dbg!(&assets_test_ignore_dir);
    let manager = super::ConfigManager::new(true, true);
    let is_ignored = manager.is_ignored_config_file(&assets_test_ignore_dir);
    dbg!(is_ignored);
    assert!(is_ignored);
  }
}
