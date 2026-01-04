#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "ffi")]
use safer_ffi::prelude::*;

/// 初始化 panic hook
///
/// **导出目标**: SO
#[cfg(feature = "ffi")]
#[ffi_export]
pub fn novelsaga_init() {
  std::panic::set_hook(Box::new(|panic_info| {
    eprintln!("NovelSaga panic: {:?}", panic_info);
  }));
}

/// 获取版本号
///
/// **导出目标**: SO
#[cfg(feature = "ffi")]
#[ffi_export]
pub fn novelsaga_version() -> char_p::Box {
  env!("CARGO_PKG_VERSION").to_string().try_into().unwrap()
}

pub mod article;
pub mod config;
pub mod library;
pub mod state;
