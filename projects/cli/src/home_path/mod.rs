use std::{path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;

use crate::args::GLOBAL_CLI;

#[allow(dead_code)]
pub static PROJECT_DIRS: LazyLock<Option<ProjectDirs>> =
  LazyLock::new(|| ProjectDirs::from("com", "Novel Saga", "NovelSaga"));

#[allow(dead_code)]
pub static PLUGINS_SEARCH_PATH: LazyLock<Vec<PathBuf>> = LazyLock::new(|| {
  let mut paths: Vec<PathBuf> = Vec::new();
  if let Some(proj_dirs) = &*PROJECT_DIRS {
    let conf_dir = proj_dirs.config_dir();
    paths.push(conf_dir.join("plugins"));
  }
  let external_search_path = &*GLOBAL_CLI.external_plugin_search_path.clone().unwrap_or_default();
  paths.extend_from_slice(external_search_path);
  paths
});
