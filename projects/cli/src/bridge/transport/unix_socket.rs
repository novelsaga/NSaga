//! Unix Domain Socket 传输层实现

use std::{
  io::{BufRead, BufReader, Write},
  os::unix::net::UnixStream,
  path::Path,
  time::Duration,
};

use super::Transport;
use crate::bridge::{
  error::{BridgeError, Result},
  rpc::types::{RpcRequest, RpcResponse},
};

/// Unix Domain Socket 传输层实现
///
/// 用于与 LSP 服务器或其他本地服务通过 Unix Socket 通信
/// 支持读写超时和连接管理
pub struct UnixSocketTransport {
  stream: UnixStream,
  reader: BufReader<UnixStream>,
  closed: bool,
  timeout: Duration,
}

impl UnixSocketTransport {
  /// 连接到指定的 Unix Socket
  ///
  /// # Arguments
  ///
  /// * `socket_path` - Unix Socket 文件路径
  ///
  /// # Errors
  ///
  /// 如果连接失败返回错误
  pub fn connect(socket_path: &Path) -> Result<Self> {
    let stream = UnixStream::connect(socket_path).map_err(|e| BridgeError::IoError {
      context: format!("Failed to connect to Unix socket at {}", socket_path.display()),
      source: e,
    })?;

    let timeout = Duration::from_secs(30);

    // 设置读超时
    stream
      .set_read_timeout(Some(timeout))
      .map_err(|e| BridgeError::IoError {
        context: "Failed to set read timeout on Unix socket".to_string(),
        source: e,
      })?;

    // 设置写超时
    stream
      .set_write_timeout(Some(timeout))
      .map_err(|e| BridgeError::IoError {
        context: "Failed to set write timeout on Unix socket".to_string(),
        source: e,
      })?;

    // 克隆流用于读取
    let reader_stream = stream.try_clone().map_err(|e| BridgeError::IoError {
      context: "Failed to clone Unix socket for reading".to_string(),
      source: e,
    })?;

    let reader = BufReader::new(reader_stream);

    Ok(Self {
      stream,
      reader,
      closed: false,
      timeout,
    })
  }

  /// 设置接收/发送超时时间
  ///
  /// # Arguments
  ///
  /// * `timeout` - 超时时间
  #[must_use]
  pub fn with_timeout(mut self, timeout: Duration) -> Self {
    self.timeout = timeout;
    // 尝试在流上设置超时（忽略错误以支持构建器模式）
    let _ = self.stream.set_read_timeout(Some(timeout));
    let _ = self.stream.set_write_timeout(Some(timeout));
    self
  }
}

impl Transport for UnixSocketTransport {
  fn send(&mut self, message: &RpcRequest) -> Result<()> {
    if self.closed {
      return Err(BridgeError::TransportClosed);
    }

    let json = message.to_json_string()?;
    writeln!(self.stream, "{json}").map_err(|e| BridgeError::IoError {
      context: "Failed to write JSON-RPC request to Unix socket".to_string(),
      source: e,
    })?;

    self.stream.flush().map_err(|e| BridgeError::IoError {
      context: "Failed to flush Unix socket after writing request".to_string(),
      source: e,
    })?;

    Ok(())
  }

  fn receive(&mut self) -> Result<RpcResponse> {
    if self.closed {
      return Err(BridgeError::TransportClosed);
    }

    let mut line = String::new();
    loop {
      line.clear();
      match self.reader.read_line(&mut line) {
        Ok(0) => {
          // EOF - 远程端已关闭连接
          self.closed = true;
          return Err(BridgeError::TransportClosed);
        }
        Ok(_) => {
          let trimmed = line.trim();

          // 跳过空行
          if trimmed.is_empty() {
            // 继续读取下一行
            continue;
          }

          // 跳过非 JSON 行
          if !trimmed.starts_with('{') {
            eprintln!("[UnixSocketTransport] Skipping non-JSON line: {trimmed}");
            // 继续读取下一行
            continue;
          }

          // 解析 JSON-RPC 响应
          match RpcResponse::from_json_str(trimmed) {
            Ok(response) => return Ok(response),
            Err(e) => {
              eprintln!("[UnixSocketTransport] Failed to parse JSON-RPC response: {e}");
              eprintln!("[UnixSocketTransport] Raw line: {trimmed}");
              return Err(BridgeError::JsonParseError(e));
            }
          }
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
          // WouldBlock 表示暂时无数据，继续等待
        }
        Err(e) => {
          self.closed = true;
          return Err(BridgeError::IoError {
            context: "Failed to read JSON-RPC response from Unix socket".to_string(),
            source: e,
          });
        }
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
  // 测试将需要真实的 Unix Socket 设置，暂留作为占位符
}
