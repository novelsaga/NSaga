use std::{
  ffi::{CStr, CString},
  os::raw::c_char,
};

use novelsaga_core::{article::Article, config::formatter::FormatConfig, library::formatter::format_text};

/// Android JNI 导出 - 格式化文本
///
/// # Safety
/// 调用者必须确保传入的指针有效
#[unsafe(no_mangle)]
pub unsafe extern "C" fn novelsaga_format_text(
  content: *const c_char,
  indent_spaces: usize,
  blank_lines: usize,
) -> *mut c_char {
  // 输入验证
  if content.is_null() {
    return std::ptr::null_mut();
  }

  // 转换输入
  let Ok(content_str) = unsafe { CStr::from_ptr(content) }.to_str() else {
    return std::ptr::null_mut();
  };

  // 创建配置
  let config = FormatConfig {
    indent_spaces,
    blank_lines_between_paragraphs: blank_lines,
  };

  // 创建文章并格式化
  let article = Article::new(content_str.to_string());
  let result_article = format_text(&article, &config);
  let result = result_article.content_ref().to_string();

  // 返回结果
  match CString::new(result) {
    Ok(s) => s.into_raw(),
    Err(_) => std::ptr::null_mut(),
  }
}

/// 释放字符串内存
///
/// # Safety
/// 必须传入由 ``novelsaga_format_text`` 返回的指针
#[unsafe(no_mangle)]
pub unsafe extern "C" fn novelsaga_free_string(ptr: *mut c_char) {
  if !ptr.is_null() {
    let _ = unsafe { CString::from_raw(ptr) };
  }
}

/// 获取版本信息
#[unsafe(no_mangle)]
pub extern "C" fn novelsaga_version() -> *const c_char {
  concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast::<c_char>()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_android_ffi() {
    unsafe {
      let content = CString::new("你好世界").unwrap();

      let result = novelsaga_format_text(content.as_ptr(), 2, 1);

      assert!(!result.is_null());

      let result_str = CStr::from_ptr(result).to_string_lossy();
      assert!(result_str.contains("你好"));

      novelsaga_free_string(result);
    }
  }

  #[test]
  fn test_version() {
    let version = novelsaga_version();
    unsafe {
      let version_str = CStr::from_ptr(version).to_string_lossy();
      assert!(!version_str.is_empty());
    }
  }
}
