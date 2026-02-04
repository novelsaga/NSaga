//! 基于 stdin/stdout 的传输层实现

use std::{
  io::{BufRead, BufReader, BufWriter, Write},
  process::{Child, ChildStdin, ChildStdout},
  sync::{
    Arc, Mutex,
    mpsc::{RecvTimeoutError, channel},
  },
  thread,
  time::Duration,
};

use super::Transport;
use crate::bridge::{
  error::{BridgeError, Result},
  rpc::types::{RpcRequest, RpcResponse},
};

/// 基于 stdin/stdout 的传输层实现
///
/// 用于与子进程通信，通过 stdin 发送 JSON-RPC 请求，
/// 通过 stdout 接收 JSON-RPC 响应
pub struct StdioTransport {
  stdin: Arc<Mutex<Box<dyn Write + Send>>>,
  stdout: Arc<Mutex<BufReader<ChildStdout>>>,
  closed: bool,
  timeout: Duration,
}

impl StdioTransport {
  /// 从子进程的 stdin 和 stdout 创建传输层
  #[must_use]
  pub fn new(stdin: ChildStdin, stdout: ChildStdout) -> Self {
    Self {
      stdin: Arc::new(Mutex::new(Box::new(stdin))),
      stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
      closed: false,
      timeout: Duration::from_secs(30), // 默认 30 秒超时
    }
  }

  /// 设置接收超时时间
  #[must_use]
  pub fn with_timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    self
  }

  /// 从子进程引用创建传输层
  ///
  /// 注意：会 take stdin 和 stdout，子进程的这两个字段将变为 None
  ///
  /// # Errors
  ///
  /// 如果子进程的 stdin 或 stdout 已被 take 则返回错误
  pub fn from_child(child: &mut Child) -> Result<Self> {
    let stdin = child.stdin.take().ok_or_else(|| {
       BridgeError::Other(
         "Cannot get stdin from child process\n\nCause: stdin may have been taken by other code\n\nSolution: Check if from_child() is called multiple times".into(),
       )
     })?;
    let stdout = child.stdout.take().ok_or_else(|| {
       BridgeError::Other(
         "Cannot get stdout from child process\n\nCause: stdout may have been taken by other code\n\nSolution: Check if from_child() is called multiple times"
           .into(),
       )
     })?;
    Ok(Self::new(stdin, stdout))
  }

  /// 从已经包装的 BufWriter/BufReader 创建传输层
  ///
  /// 用于 `RuntimeProcess::take_io()` 返回的已缓冲 IO
  pub fn from_buffered(mut stdin: BufWriter<ChildStdin>, stdout: BufReader<ChildStdout>) -> Result<Self> {
    // 刷新并解包 stdin，直接持有 ChildStdin，避免 BufWriter 在 drop 时的问题
    stdin.flush().map_err(|e| BridgeError::IoError {
      context: "Failed to flush buffered stdin".to_string(),
      source: e,
    })?;
    let inner_stdin = stdin.into_inner().map_err(|e| BridgeError::IoError {
      context: "Failed to unwrap BufWriter from stdin".to_string(),
      source: e.into_error(),
    })?;

    Ok(Self {
      stdin: Arc::new(Mutex::new(Box::new(inner_stdin))),
      stdout: Arc::new(Mutex::new(stdout)),
      closed: false,
      timeout: Duration::from_secs(30),
    })
  }
}

impl Transport for StdioTransport {
  fn send(&mut self, message: &RpcRequest) -> Result<()> {
    if self.closed {
      return Err(BridgeError::TransportClosed);
    }

    let json = message.to_json_string()?;
    let mut stdin = self.stdin.lock().unwrap();

    // 写入 JSON + 换行符
    writeln!(stdin, "{json}").map_err(|e| BridgeError::IoError {
      context: "Failed to write JSON-RPC request to process stdin".to_string(),
      source: e,
    })?;
    stdin.flush().map_err(|e| BridgeError::IoError {
      context: "Failed to flush process stdin after writing request".to_string(),
      source: e,
    })?;

    Ok(())
  }

  fn receive(&mut self) -> Result<RpcResponse> {
    if self.closed {
      return Err(BridgeError::TransportClosed);
    }

    let timeout = self.timeout;
    let stdout = Arc::clone(&self.stdout);
    let (tx, rx) = channel();

    // 在新线程中读取，避免阻塞
    thread::spawn(move || {
      let mut stdout = stdout.lock().unwrap();
      let mut line = String::new();

      loop {
        line.clear();
        match stdout.read_line(&mut line) {
          Ok(0) => {
            // EOF
            let _ = tx.send(Err(BridgeError::TransportClosed));
            return;
          }
          Ok(_) => {
            let trimmed = line.trim();

            // 跳过空行
            if trimmed.is_empty() {
              continue;
            }

            // 跳过非 JSON 行
            if !trimmed.starts_with('{') {
              eprintln!("[StdioTransport] Skipping non-JSON line: {trimmed}");
              continue;
            }

            // 解析 JSON-RPC 响应
            match RpcResponse::from_json_str(trimmed) {
              Ok(response) => {
                let _ = tx.send(Ok(response));
                return;
              }
              Err(e) => {
                eprintln!("[StdioTransport] Failed to parse JSON-RPC response: {e}");
                eprintln!("[StdioTransport] Raw line: {trimmed}");
                let _ = tx.send(Err(BridgeError::JsonParseError(e)));
                return;
              }
            }
          }
          Err(e) => {
            let _ = tx.send(Err(BridgeError::IoError {
              context: "Failed to read JSON-RPC response from process stdout".to_string(),
              source: e,
            }));
            return;
          }
        }
      }
    });

    // 等待结果或超时
    match rx.recv_timeout(timeout) {
      Ok(result) => {
        if result.is_err() {
          self.closed = true;
        }
        result
      }
      Err(RecvTimeoutError::Timeout) => {
        self.closed = true;
        Err(BridgeError::Timeout(timeout))
      }
      Err(RecvTimeoutError::Disconnected) => {
        self.closed = true;
        Err(BridgeError::TransportClosed)
      }
    }
  }

  fn close(&mut self) -> Result<()> {
    self.closed = true;
    Ok(())
  }

  fn is_closed(&self) -> bool {
    self.closed
  }
}

#[cfg(test)]
mod tests {
  // 测试将在后续添加，需要真实的子进程
}
