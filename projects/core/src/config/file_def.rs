use std::sync::OnceLock;

use config::FileStoredFormat;

static BASE_CONFIG_FILE_EXTENSIONS: OnceLock<&'static [&'static str]> = OnceLock::new();

/// # Panics
pub fn get_base_config_file_extensions() -> &'static [&'static str] {
  BASE_CONFIG_FILE_EXTENSIONS.get_or_init(|| {
    use config::FileFormat;
    let file_formats = [
      FileFormat::Corn,
      FileFormat::Ron,
      FileFormat::Toml,
      FileFormat::Yaml,
      FileFormat::Json5,
      FileFormat::Json,
      FileFormat::Ini,
    ];
    // map to extensions
    let extensions: Vec<&'static str> = file_formats
      .iter()
      .flat_map(FileStoredFormat::file_extensions)
      .copied()
      .collect();
    Box::leak(extensions.into_boxed_slice())
  });
  BASE_CONFIG_FILE_EXTENSIONS.get().unwrap()
}

/// 配置文件名(不含扩展名)
pub const CONFIG_FILE_NAMES: &[&str] = &["novelsaga.config", ".novelsaga"];

pub const IGNORE_CONFIG_FILE_NAMES: &[&str] = &[".novelsagaignore", ".nsignore", ".novelsaga.ignore", ".ns.ignore"];
