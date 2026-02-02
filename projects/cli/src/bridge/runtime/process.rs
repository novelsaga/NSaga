//! JavaScript 运行时进程管理
//!
//! 负责启动、监控和管理 JavaScript 运行时进程

use std::{
  collections::HashMap,
  io::{BufReader, BufWriter, Write},
  path::Path,
  process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use crate::bridge::{
  error::{BridgeError, Result},
  runtime::{RuntimeInfo, RuntimeType},
};

/// JavaScript 运行时进程
pub struct RuntimeProcess {
  /// 运行时信息
  runtime_info: RuntimeInfo,
  /// 子进程句柄
  child: Child,
  /// stdin 写入器
  stdin: Option<BufWriter<ChildStdin>>,
  /// stdout 读取器
  stdout: Option<BufReader<ChildStdout>>,
}

impl RuntimeProcess {
  /// 启动运行时进程
  ///
  /// # 参数
  ///
  /// * `runtime_info` - 运行时信息
  /// * `script_path` - 要执行的脚本路径
  /// * `env` - 环境变量
  ///
  /// # 示例
  ///
  /// ```rust,no_run
  /// use std::collections::HashMap;
  /// use std::path::Path;
  ///
  /// let runtime_info = RuntimeInfo {
  ///     runtime_type: RuntimeType::NodeJs,
  ///     path: PathBuf::from("/usr/bin/node"),
  ///     version: "v20.0.0".to_string(),
  /// };
  ///
  /// let mut env = HashMap::new();
  /// env.insert(env_keys::CONFIG_PATH.to_string(), "/path/to/config.js".to_string());
  ///
  /// let process = RuntimeProcess::spawn(runtime_info, Path::new("bridge.js"), env)?;
  /// ```
  pub fn spawn(runtime_info: RuntimeInfo, script_path: &Path, env: HashMap<String, String>) -> Result<Self> {
    // 构建命令
    let mut cmd = Command::new(&runtime_info.path);

    // 添加运行参数
    let args = runtime_info.get_run_args(script_path);
    cmd.args(&args);

    // 设置环境变量
    for (key, value) in env {
      cmd.env(key, value);
    }

    // 设置运行时标识环境变量（用于 config-bridge 等动态选择适配层）
    let runtime_name = match runtime_info.runtime_type {
      RuntimeType::NodeJs => "node",
      RuntimeType::Bun => "bun",
      RuntimeType::Deno => "deno",
    };
    cmd.env(crate::bridge::env_keys::RUNTIME, runtime_name);

    // 配置 stdio
    cmd
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit()); // stderr 继承，用于调试输出

    // 启动进程
    let mut child = cmd.spawn().map_err(|e| {
      BridgeError::Other(format!(
        "Failed to spawn {} process: {}",
        runtime_info.runtime_type.executable_name(),
        e
      ))
    })?;

    // 获取 stdin/stdout handles
    let stdin = child.stdin.take().map(BufWriter::new);
    let stdout = child.stdout.take().map(BufReader::new);

    if stdin.is_none() || stdout.is_none() {
      // 清理进程
      let _ = child.kill();
      return Err(BridgeError::Other(
        "Failed to capture stdin/stdout from child process".to_string(),
      ));
    }

    Ok(Self {
      runtime_info,
      child,
      stdin,
      stdout,
    })
  }

  /// 获取运行时信息
  pub fn runtime_info(&self) -> &RuntimeInfo {
    &self.runtime_info
  }

  /// 获取进程 ID
  pub fn pid(&self) -> u32 {
    self.child.id()
  }

  /// 检查进程是否仍在运行
  pub fn is_running(&mut self) -> bool {
    match self.child.try_wait() {
      Ok(None) => true, // 进程仍在运行
      // Process exited or error - assume not running
      Ok(Some(_)) | Err(_) => false,
    }
  }

  /// 获取 stdin 写入器的可变引用
  pub fn stdin_mut(&mut self) -> Option<&mut BufWriter<ChildStdin>> {
    self.stdin.as_mut()
  }

  /// 获取 stdout 读取器的可变引用
  pub fn stdout_mut(&mut self) -> Option<&mut BufReader<ChildStdout>> {
    self.stdout.as_mut()
  }

  /// 取出 stdin 和 stdout（ownership transfer）
  ///
  /// 返回 `(stdin, stdout)`，如果已被取出则返回 None
  pub fn take_io(&mut self) -> Option<(BufWriter<ChildStdin>, BufReader<ChildStdout>)> {
    match (self.stdin.take(), self.stdout.take()) {
      (Some(stdin), Some(stdout)) => Some((stdin, stdout)),
      _ => None,
    }
  }

  /// 向进程发送数据
  pub fn send(&mut self, data: &str) -> Result<()> {
    if let Some(stdin) = &mut self.stdin {
      stdin.write_all(data.as_bytes()).map_err(BridgeError::IoError)?;
      stdin.write_all(b"\n").map_err(BridgeError::IoError)?;
      stdin.flush().map_err(BridgeError::IoError)?;
      Ok(())
    } else {
      Err(BridgeError::TransportClosed)
    }
  }

  /// 优雅关闭进程
  ///
  /// 尝试发送 SIGTERM，等待进程自然退出
  /// 如果超时，则强制 kill
  pub fn shutdown(&mut self) -> Result<()> {
    if !self.is_running() {
      return Ok(());
    }

    // 关闭 stdin，通知进程
    drop(self.stdin.take());

    // 等待进程退出（最多 5 秒）
    for _ in 0..50 {
      if !self.is_running() {
        return Ok(());
      }
      std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // 超时，强制 kill
    self.child.kill().map_err(BridgeError::IoError)?;
    self.child.wait().map_err(BridgeError::IoError)?;

    Ok(())
  }

  /// 强制杀死进程
  pub fn kill(&mut self) -> Result<()> {
    if self.is_running() {
      self.child.kill().map_err(BridgeError::IoError)?;
      self.child.wait().map_err(BridgeError::IoError)?;
    }
    Ok(())
  }

  /// 等待进程退出
  pub fn wait(&mut self) -> Result<()> {
    self.child.wait().map_err(BridgeError::IoError)?;
    Ok(())
  }

  /// `检查进程是否存活（is_running` 的别名，用于一致性）
  pub fn is_alive(&mut self) -> bool {
    self.is_running()
  }

  /// 获取底层 Child 进程的可变引用
  pub fn child_mut(&mut self) -> &mut Child {
    &mut self.child
  }
}

impl Drop for RuntimeProcess {
  fn drop(&mut self) {
    // 确保进程被清理
    let _ = self.kill();
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::bridge::runtime::{RuntimeDiscovery, RuntimeType};

  #[test]
  #[ignore = "requires actual runtime environment"]
  fn test_spawn_and_kill() {
    let discovery = RuntimeDiscovery::new();
    let runtime = discovery
      .find_runtime(RuntimeType::NodeJs)
      .expect("Failed to discover")
      .expect("Node.js not found");

    // 创建一个简单的测试脚本
    let test_script = std::env::temp_dir().join("test_process.js");
    std::fs::write(&test_script, "console.log('Hello'); process.stdin.resume();").expect("Failed to write test script");

    let env = HashMap::new();
    let mut process = RuntimeProcess::spawn(runtime, &test_script, env).expect("Failed to spawn");

    assert!(process.is_running());

    process.kill().expect("Failed to kill");
    assert!(!process.is_running());

    // 清理
    std::fs::remove_file(test_script).ok();
  }

  #[test]
  #[ignore = "requires actual runtime environment"]
  fn test_send_data() {
    let discovery = RuntimeDiscovery::new();
    let runtime = discovery
      .find_runtime(RuntimeType::NodeJs)
      .expect("Failed to discover")
      .expect("Node.js not found");

    // 创建一个回显脚本
    let test_script = std::env::temp_dir().join("test_echo.js");
    std::fs::write(
      &test_script,
      r"
        const readline = require('readline');
        const rl = readline.createInterface({ input: process.stdin });
        rl.on('line', line => console.log('Received: ' + line));
      ",
    )
    .expect("Failed to write test script");

    let env = HashMap::new();
    let mut process = RuntimeProcess::spawn(runtime, &test_script, env).expect("Failed to spawn");

    // 发送数据
    process.send("Hello").expect("Failed to send");

    std::thread::sleep(std::time::Duration::from_millis(100));

    process.kill().expect("Failed to kill");

    // 清理
    std::fs::remove_file(test_script).ok();
  }

  #[test]
  #[ignore = "requires actual runtime environment"]
  fn test_shutdown_graceful() {
    let discovery = RuntimeDiscovery::new();
    let runtime = discovery
      .find_runtime(RuntimeType::NodeJs)
      .expect("Failed to discover")
      .expect("Node.js not found");

    // 创建一个监听 stdin 关闭的脚本
    let test_script = std::env::temp_dir().join("test_graceful.js");
    std::fs::write(
      &test_script,
      r"
        process.stdin.on('end', () => {
          console.log('stdin closed, exiting...');
          process.exit(0);
        });
        process.stdin.resume();
      ",
    )
    .expect("Failed to write test script");

    let env = HashMap::new();
    let mut process = RuntimeProcess::spawn(runtime, &test_script, env).expect("Failed to spawn");

    assert!(process.is_running());

    // 优雅关闭
    process.shutdown().expect("Failed to shutdown");

    assert!(!process.is_running());

    // 清理
    std::fs::remove_file(test_script).ok();
  }
}
