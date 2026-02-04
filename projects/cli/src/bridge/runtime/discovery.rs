//! JavaScript 运行时检测
//!
//! 支持检测 Node.js, Bun, Deno 运行时及其版本

use std::{
  path::{Path, PathBuf},
  process::Command,
};

use crate::bridge::error::{BridgeError, Result};

/// JavaScript 运行时类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeType {
  /// Node.js 运行时
  NodeJs,
  /// Bun 运行时
  Bun,
  /// Deno 运行时
  Deno,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
impl RuntimeType {
  /// 获取运行时可执行文件名（不含扩展名）
  pub fn executable_name(&self) -> &str {
    match self {
      RuntimeType::NodeJs => "node",
      RuntimeType::Bun => "bun",
      RuntimeType::Deno => "deno",
    }
  }

  /// 获取运行时的完整可执行文件名（跨平台）
  #[cfg(windows)]
  pub fn executable_filename(&self) -> String {
    format!("{}.exe", self.executable_name())
  }

  #[cfg(not(windows))]
  pub fn executable_filename(&self) -> String {
    self.executable_name().to_string()
  }
}

/// 运行时信息
#[derive(Debug, Clone)]
pub struct RuntimeInfo {
  /// 运行时类型
  pub runtime_type: RuntimeType,
  /// 可执行文件路径
  pub path: PathBuf,
  /// 版本号
  pub version: String,
}

impl RuntimeInfo {
  /// 检测 TypeScript 是否原生支持
  ///
  /// Node.js ≥23.6 支持原生 TypeScript（通过 --experimental-strip-types）
  pub fn supports_native_typescript(&self) -> bool {
    match self.runtime_type {
      RuntimeType::NodeJs => {
        // 解析版本号，检查是否 ≥23.6
        if let Some(version) = self.version.strip_prefix('v')
          && let Some((major, rest)) = version.split_once('.')
          && let Ok(major_num) = major.parse::<u32>()
        {
          if major_num > 23 {
            return true;
          }
          if major_num == 23
            && let Some((minor, _)) = rest.split_once('.')
            && let Ok(minor_num) = minor.parse::<u32>()
          {
            return minor_num >= 6;
          }
        }
        false
      }
      // Bun and Deno both natively support TypeScript
      RuntimeType::Bun | RuntimeType::Deno => true,
    }
  }

  /// 获取运行脚本的参数
  ///
  /// 不同运行时运行脚本的方式不同：
  /// - Node.js: `node script.js` 或 `node --experimental-strip-types script.ts`
  /// - Bun: `bun run script.js`
  /// - Deno: `deno run script.js`
  pub fn get_run_args(&self, script_path: &Path) -> Vec<String> {
    let mut args = Vec::new();

    match self.runtime_type {
      RuntimeType::NodeJs => {
        // Node.js ≥23.6 支持 TypeScript
        if (script_path.extension().and_then(|s| s.to_str()) == Some("ts")
          || script_path.extension().and_then(|s| s.to_str()) == Some("mts")
          || script_path.extension().and_then(|s| s.to_str()) == Some("cts"))
          && self.supports_native_typescript()
        {
          args.push("--experimental-strip-types".to_string());
        }
        args.push(script_path.to_string_lossy().to_string());
      }
      RuntimeType::Bun => {
        args.push("run".to_string());
        args.push(script_path.to_string_lossy().to_string());
      }
      RuntimeType::Deno => {
        args.push("run".to_string());
        args.push("--allow-read".to_string());
        args.push("--allow-env".to_string());
        args.push(script_path.to_string_lossy().to_string());
      }
    }

    args
  }
}

/// 运行时发现器
#[derive(Clone, Copy)]
pub struct RuntimeDiscovery;

#[allow(clippy::unused_self, clippy::trivially_copy_pass_by_ref)]
impl RuntimeDiscovery {
  /// 创建运行时发现器
  pub fn new() -> Self {
    Self
  }

  /// 在 PATH 中查找指定运行时
  ///
  /// 如果提供了 `user_path`，则优先使用用户指定的路径
  pub fn find_runtime(&self, runtime_type: RuntimeType) -> Result<Option<RuntimeInfo>> {
    self.find_runtime_with_path(runtime_type, None)
  }

  /// 使用用户指定的路径查找运行时
  ///
  /// # 参数
  ///
  /// * `runtime_type` - 运行时类型
  /// * `user_path` - 用户指定的可执行文件路径（可选）
  pub fn find_runtime_with_path(
    &self,
    runtime_type: RuntimeType,
    user_path: Option<PathBuf>,
  ) -> Result<Option<RuntimeInfo>> {
    // 如果用户提供了路径，优先使用
    if let Some(path) = user_path {
      if let Some(version) = Self::get_version(&path)? {
        return Ok(Some(RuntimeInfo {
          runtime_type,
          path,
          version,
        }));
      }
      // 用户指定的路径无效
      return Ok(None);
    }

    let executable = runtime_type.executable_filename();

    // 尝试使用 which/where 命令查找
    #[cfg(not(windows))]
    let which_cmd = "which";
    #[cfg(windows)]
    let which_cmd = "where";

    let output = Command::new(which_cmd).arg(&executable).output();

    if let Ok(output) = output
      && output.status.success()
    {
      let path_str = String::from_utf8_lossy(&output.stdout);
      let path = path_str.lines().next().unwrap_or("").trim();

      if !path.is_empty() {
        let path_buf = PathBuf::from(path);
        if let Some(version) = Self::get_version(&path_buf)? {
          return Ok(Some(RuntimeInfo {
            runtime_type,
            path: path_buf,
            version,
          }));
        }
      }
    }

    // 如果 which/where 失败，尝试直接执行（依赖 PATH）
    let executable_path = PathBuf::from(&executable);
    if let Some(version) = Self::get_version(&executable_path)? {
      return Ok(Some(RuntimeInfo {
        runtime_type,
        path: executable_path,
        version,
      }));
    }

    Ok(None)
  }

  /// 查找所有可用的运行时
  pub fn find_all_runtimes(&self) -> Result<Vec<RuntimeInfo>> {
    let mut runtimes = Vec::new();

    for runtime_type in [RuntimeType::NodeJs, RuntimeType::Bun, RuntimeType::Deno] {
      if let Some(info) = self.find_runtime(runtime_type)? {
        runtimes.push(info);
      }
    }

    Ok(runtimes)
  }

  /// 查找最佳运行时（优先级：Node.js > Bun > Deno）
  pub fn find_best_runtime(&self) -> Result<RuntimeInfo> {
    // 优先选择 Node.js
    if let Some(info) = self.find_runtime(RuntimeType::NodeJs)? {
      return Ok(info);
    }

    // 其次选择 Bun
    if let Some(info) = self.find_runtime(RuntimeType::Bun)? {
      return Ok(info);
    }

    // 最后选择 Deno
    if let Some(info) = self.find_runtime(RuntimeType::Deno)? {
      return Ok(info);
    }

    Err(BridgeError::Other(
       "未找到任何 JavaScript 运行时\n\n搜索的运行时:\n  - Node.js (https://nodejs.org/)\n  - Bun (https://bun.sh/)\n  - Deno (https://deno.land/)\n\n解决方案:\n  1. 安装上述任意一个运行时\n  2. 确保运行时在 $PATH 中\n  3. 或使用 --runtime 和 --{runtime}-path 指定自定义路径".to_string(),
     ))
  }

  /// 根据用户偏好查找运行时
  ///
  /// # 参数
  ///
  /// * `preferred_runtime` - 用户偏好的运行时类型（可选）
  /// * `user_path` - 用户指定的可执行文件路径（可选）
  ///
  /// # 返回
  ///
  /// 如果指定了 `preferred_runtime`，则只尝试该运行时。
  /// 如果未指定，则回退到 `find_best_runtime()`。
  pub fn find_runtime_with_preference(
    &self,
    preferred_runtime: Option<RuntimeType>,
    user_path: Option<PathBuf>,
  ) -> Result<RuntimeInfo> {
    if let Some(runtime_type) = preferred_runtime {
      // 用户指定了运行时，只尝试该运行时
      let runtime_name = match runtime_type {
        RuntimeType::NodeJs => "Node.js",
        RuntimeType::Bun => "Bun",
        RuntimeType::Deno => "Deno",
      };

      let user_path_str = user_path
        .as_ref()
        .map(|p| format!("{}", p.display()))
        .unwrap_or_default();
      self.find_runtime_with_path(runtime_type, user_path)?.ok_or_else(|| {
         BridgeError::RuntimeNotFound {
           runtime_type: runtime_name.to_string(),
           searched_paths: format!("  - $PATH\n  - 用户指定: {user_path_str}"),
           suggestion: format!(
             "1. 安装 {runtime_name}\n   - Node.js: https://nodejs.org/\n   - Bun: https://bun.sh/\n   - Deno: https://deno.land/\n2. 确保在 $PATH 中\n3. 或使用 --{} 指定路径",
             runtime_name.to_lowercase()
           ),
         }
       })
    } else {
      // 未指定运行时，使用自动检测
      self.find_best_runtime()
    }
  }

  /// 获取运行时版本
  fn get_version(path: &Path) -> Result<Option<String>> {
    let output = Command::new(path).arg("--version").output();

    match output {
      Ok(output) if output.status.success() => {
        let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(Some(version_str))
      }
      Ok(_) => Ok(None),
      Err(e) => {
        // 如果是 NotFound 错误，返回 None 而不是错误
        if e.kind() == std::io::ErrorKind::NotFound {
          Ok(None)
        } else {
          Err(BridgeError::IoError {
            context: format!("Failed to get runtime version from {}", path.display()),
            source: e,
          })
        }
      }
    }
  }
}

impl Default for RuntimeDiscovery {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_runtime_type_executable_name() {
    assert_eq!(RuntimeType::NodeJs.executable_name(), "node");
    assert_eq!(RuntimeType::Bun.executable_name(), "bun");
    assert_eq!(RuntimeType::Deno.executable_name(), "deno");
  }

  #[test]
  #[cfg(windows)]
  fn test_runtime_type_executable_filename_windows() {
    assert_eq!(RuntimeType::NodeJs.executable_filename(), "node.exe");
    assert_eq!(RuntimeType::Bun.executable_filename(), "bun.exe");
    assert_eq!(RuntimeType::Deno.executable_filename(), "deno.exe");
  }

  #[test]
  #[cfg(not(windows))]
  fn test_runtime_type_executable_filename_unix() {
    assert_eq!(RuntimeType::NodeJs.executable_filename(), "node");
    assert_eq!(RuntimeType::Bun.executable_filename(), "bun");
    assert_eq!(RuntimeType::Deno.executable_filename(), "deno");
  }

  #[test]
  fn test_supports_native_typescript() {
    let info_old = RuntimeInfo {
      runtime_type: RuntimeType::NodeJs,
      path: PathBuf::from("/usr/bin/node"),
      version: "v20.0.0".to_string(),
    };
    assert!(!info_old.supports_native_typescript());

    let info_new = RuntimeInfo {
      runtime_type: RuntimeType::NodeJs,
      path: PathBuf::from("/usr/bin/node"),
      version: "v23.6.0".to_string(),
    };
    assert!(info_new.supports_native_typescript());

    let info_newer = RuntimeInfo {
      runtime_type: RuntimeType::NodeJs,
      path: PathBuf::from("/usr/bin/node"),
      version: "v24.0.0".to_string(),
    };
    assert!(info_newer.supports_native_typescript());

    let info_bun = RuntimeInfo {
      runtime_type: RuntimeType::Bun,
      path: PathBuf::from("/usr/bin/bun"),
      version: "1.0.0".to_string(),
    };
    assert!(info_bun.supports_native_typescript());
  }

  #[test]
  fn test_get_run_args() {
    let node_info = RuntimeInfo {
      runtime_type: RuntimeType::NodeJs,
      path: PathBuf::from("/usr/bin/node"),
      version: "v23.6.0".to_string(),
    };

    // JavaScript 文件
    let js_args = node_info.get_run_args(Path::new("script.js"));
    assert_eq!(js_args, vec!["script.js"]);

    // TypeScript 文件（支持原生 TS）
    let ts_args = node_info.get_run_args(Path::new("script.ts"));
    assert_eq!(ts_args, vec!["--experimental-strip-types", "script.ts"]);

    // Bun
    let bun_info = RuntimeInfo {
      runtime_type: RuntimeType::Bun,
      path: PathBuf::from("/usr/bin/bun"),
      version: "1.0.0".to_string(),
    };
    let bun_args = bun_info.get_run_args(Path::new("script.js"));
    assert_eq!(bun_args, vec!["run", "script.js"]);

    // Deno
    let deno_info = RuntimeInfo {
      runtime_type: RuntimeType::Deno,
      path: PathBuf::from("/usr/bin/deno"),
      version: "2.0.0".to_string(),
    };
    let deno_args = deno_info.get_run_args(Path::new("script.js"));
    assert_eq!(deno_args, vec!["run", "--allow-read", "--allow-env", "script.js"]);
  }
}
