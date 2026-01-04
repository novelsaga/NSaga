//! SO FFI 辅助模块
//!
//! 仅用于生成 C 头文件

/// 生成 C 头文件
#[cfg(feature = "headers")]
pub fn generate_headers() -> std::io::Result<()> {
  safer_ffi::headers::builder().to_file("novelsaga_core.h")?.generate()
}
