use std::{
  collections::HashMap,
  path::PathBuf,
  process::{Command, Stdio},
};

use anyhow::{Context, Result};
use lsp_types::{
  ClientCapabilities, DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
  DocumentFormattingParams, FormattingOptions, InitializeParams, TextDocumentContentChangeEvent,
  TextDocumentIdentifier, TextDocumentItem, Url, VersionedTextDocumentIdentifier, WorkDoneProgressParams,
  notification::{DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument},
  request::{Formatting, Initialize, Request, Shutdown},
};
use serde::Serialize;
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::{
  io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
  process::{Child, ChildStdin, ChildStdout, Command as TokioCommand},
  runtime::Builder,
  time::{Duration, sleep},
};

struct JsonRpcLspClient {
  child: Child,
  stdin: ChildStdin,
  stdout: BufReader<ChildStdout>,
  next_id: i64,
}

impl JsonRpcLspClient {
  fn spawn(manifest_path: &str, cwd: &std::path::Path) -> Result<Self> {
    let mut child = TokioCommand::new("cargo")
      .args([
        "run",
        "--bin",
        "novelsaga",
        "--manifest-path",
        manifest_path,
        "--",
        "lsp",
      ])
      .current_dir(cwd)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .kill_on_drop(true)
      .spawn()
      .context("Failed to spawn novelsaga lsp")?;

    let stdin = child.stdin.take().context("Failed to capture LSP child stdin")?;
    let stdout = child.stdout.take().context("Failed to capture LSP child stdout")?;

    Ok(Self {
      child,
      stdin,
      stdout: BufReader::new(stdout),
      next_id: 1,
    })
  }

  async fn send_notification<N>(&mut self, params: N::Params) -> Result<()>
  where
    N: lsp_types::notification::Notification,
    N::Params: Serialize,
  {
    let payload = json!({
      "jsonrpc": "2.0",
      "method": N::METHOD,
      "params": params,
    });
    self.write_message(&payload).await
  }

  async fn send_request<R>(&mut self, params: R::Params) -> Result<R::Result>
  where
    R: lsp_types::request::Request,
    R::Params: Serialize,
    R::Result: serde::de::DeserializeOwned,
  {
    let id = self.next_id;
    self.next_id += 1;

    let payload = json!({
      "jsonrpc": "2.0",
      "id": id,
      "method": R::METHOD,
      "params": params,
    });

    self.write_message(&payload).await?;
    let response = self.read_response_for_id(id).await?;

    match (response.result, response.error) {
      (Some(result), None) => serde_json::from_value(result).context("Failed to deserialize JSON-RPC response result"),
      (_, Some(error)) => Err(anyhow::anyhow!(
        "JSON-RPC error response (code {}): {}",
        error.code,
        error.message
      )),
      (None, None) => Err(anyhow::anyhow!("JSON-RPC response missing both result and error")),
    }
  }

  async fn send_request_raw<P>(&mut self, method: &str, params: P) -> Result<JsonRpcResponse>
  where
    P: Serialize,
  {
    let id = self.next_id;
    self.next_id += 1;

    let payload = json!({
      "jsonrpc": "2.0",
      "id": id,
      "method": method,
      "params": params,
    });

    self.write_message(&payload).await?;
    self.read_response_for_id(id).await
  }

  async fn shutdown_and_wait(mut self) -> Result<()> {
    let status = self.child.wait().await.context("Failed to wait for LSP process exit")?;
    if !status.success() {
      return Err(anyhow::anyhow!("LSP process exited with status: {status}"));
    }
    Ok(())
  }

  async fn write_message(&mut self, value: &Value) -> Result<()> {
    let body = serde_json::to_vec(value).context("Failed to serialize JSON-RPC payload")?;
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    self
      .stdin
      .write_all(header.as_bytes())
      .await
      .context("Failed to write JSON-RPC header to LSP stdin")?;
    self
      .stdin
      .write_all(&body)
      .await
      .context("Failed to write JSON-RPC body to LSP stdin")?;
    self.stdin.flush().await.context("Failed to flush LSP stdin")?;
    Ok(())
  }

  async fn read_response_for_id(&mut self, expected_id: i64) -> Result<JsonRpcResponse> {
    loop {
      let raw = self.read_message().await?;
      let value: Value = serde_json::from_slice(&raw).context("Failed to parse JSON-RPC payload")?;

      if value.get("id").is_some() {
        let response: JsonRpcResponse = serde_json::from_value(value).context("Failed to parse JSON-RPC response")?;
        match response.id {
          Some(JsonRpcId::Number(id)) if id == expected_id => return Ok(response),
          _ => continue,
        }
      }
    }
  }

  async fn read_message(&mut self) -> Result<Vec<u8>> {
    let mut headers = HashMap::new();
    loop {
      let mut line = String::new();
      let read = self
        .stdout
        .read_line(&mut line)
        .await
        .context("Failed to read JSON-RPC header line")?;

      if read == 0 {
        return Err(anyhow::anyhow!("LSP stdout closed while reading JSON-RPC headers"));
      }

      if line == "\r\n" {
        break;
      }

      let line = line.trim_end();
      let (key, value) = line
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("Invalid JSON-RPC header line: {line}"))?;
      headers.insert(key.to_ascii_lowercase(), value.trim().to_string());
    }

    let content_length = headers
      .get("content-length")
      .ok_or_else(|| anyhow::anyhow!("Missing Content-Length header in JSON-RPC message"))?
      .parse::<usize>()
      .context("Invalid Content-Length header value")?;

    let mut body = vec![0u8; content_length];
    self
      .stdout
      .read_exact(&mut body)
      .await
      .context("Failed to read JSON-RPC message body")?;
    Ok(body)
  }
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum JsonRpcId {
  Number(i64),
  String(()),
}

#[derive(Debug, serde::Deserialize)]
struct JsonRpcError {
  code: i64,
  message: String,
}

#[derive(Debug, serde::Deserialize)]
struct JsonRpcResponse {
  id: Option<JsonRpcId>,
  result: Option<Value>,
  error: Option<JsonRpcError>,
}

pub fn run_e2e_test() -> Result<()> {
  let runtime = Builder::new_multi_thread()
    .enable_all()
    .build()
    .context("Failed to create tokio runtime for LSP E2E test")?;

  runtime.block_on(run_lsp_e2e_test())
}

pub async fn run_lsp_e2e_test() -> Result<()> {
  println!("🧪 Running LSP E2E test...");

  let temp_dir = TempDir::new().context("Failed to create temp workspace")?;
  let test_file = temp_dir.path().join("test.md");

  let original_content = "---\ntitle: Test\n---\n\nContent";
  std::fs::write(&test_file, original_content).context("Failed to create LSP test markdown file")?;

  let workspace_root = get_workspace_root()?;
  let manifest_path = workspace_root.join("projects/cli/Cargo.toml");

  let manifest_path_str = manifest_path
    .to_str()
    .context("CLI manifest path contains invalid UTF-8")?;

  let mut preflight_child = TokioCommand::new("cargo")
    .args([
      "run",
      "--bin",
      "novelsaga",
      "--manifest-path",
      manifest_path_str,
      "--",
      "lsp",
    ])
    .current_dir(temp_dir.path())
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .kill_on_drop(true)
    .spawn()
    .context("Failed to spawn novelsaga lsp with tokio::process::Command")?;

  let _preflight_stdin = preflight_child
    .stdin
    .take()
    .context("Failed to capture preflight LSP stdin")?;
  let _preflight_stdout = preflight_child
    .stdout
    .take()
    .context("Failed to capture preflight LSP stdout")?;

  sleep(Duration::from_millis(150)).await;
  preflight_child
    .kill()
    .await
    .context("Failed to terminate preflight LSP process")?;
  let _ = preflight_child.wait().await;

  let mut client = JsonRpcLspClient::spawn(manifest_path_str, temp_dir.path())?;

  let root_uri = Url::from_file_path(temp_dir.path())
    .map_err(|_| anyhow::anyhow!("Failed to convert workspace path to file URL"))?;

  let initialize_result = client
    .send_request::<Initialize>(InitializeParams {
      process_id: Some(std::process::id()),
      root_uri: Some(root_uri),
      capabilities: ClientCapabilities::default(),
      ..Default::default()
    })
    .await
    .context("initialize request failed")?;

  if initialize_result.capabilities.text_document_sync.is_none() {
    anyhow::bail!("initialize response missing textDocumentSync capability");
  }

  client
    .send_notification::<lsp_types::notification::Initialized>(lsp_types::InitializedParams {})
    .await
    .context("initialized notification failed")?;

  let file_uri =
    Url::from_file_path(&test_file).map_err(|_| anyhow::anyhow!("Failed to convert test file path to file URL"))?;

  client
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: file_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: original_content.to_string(),
      },
    })
    .await
    .context("didOpen notification failed")?;

  let changed_content = "---\ntitle: Updated\n---\n\nChanged content";

  let formatting_after_open = client
    .send_request::<Formatting>(DocumentFormattingParams {
      text_document: TextDocumentIdentifier { uri: file_uri.clone() },
      options: FormattingOptions {
        tab_size: 2,
        insert_spaces: true,
        properties: HashMap::new(),
        trim_trailing_whitespace: Some(true),
        insert_final_newline: Some(true),
        trim_final_newlines: Some(true),
      },
      work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
    })
    .await
    .context("formatting request after didOpen failed")?;

  if formatting_after_open.is_none() {
    anyhow::bail!("formatting response after didOpen was None; expected server to load opened document");
  }

  client
    .send_notification::<DidChangeTextDocument>(DidChangeTextDocumentParams {
      text_document: VersionedTextDocumentIdentifier {
        uri: file_uri.clone(),
        version: 2,
      },
      content_changes: vec![TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: changed_content.to_string(),
      }],
    })
    .await
    .context("didChange notification failed")?;

  let formatting_after_change = client
    .send_request::<Formatting>(DocumentFormattingParams {
      text_document: TextDocumentIdentifier { uri: file_uri.clone() },
      options: FormattingOptions {
        tab_size: 2,
        insert_spaces: true,
        properties: HashMap::new(),
        trim_trailing_whitespace: Some(true),
        insert_final_newline: Some(true),
        trim_final_newlines: Some(true),
      },
      work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
    })
    .await
    .context("formatting request after didChange failed")?;

  if let Some(edits) = formatting_after_change {
    let edits_json = serde_json::to_string(&edits).context("Failed to serialize formatting edits")?;
    if !edits_json.contains("Updated") {
      anyhow::bail!("formatting response after didChange does not reflect updated title; edits={edits_json}");
    }
  }

  client
    .send_notification::<DidCloseTextDocument>(DidCloseTextDocumentParams {
      text_document: TextDocumentIdentifier { uri: file_uri.clone() },
    })
    .await
    .context("didClose notification failed")?;

  let formatting_after_close = client
    .send_request_raw(
      Formatting::METHOD,
      DocumentFormattingParams {
        text_document: TextDocumentIdentifier { uri: file_uri.clone() },
        options: FormattingOptions {
          tab_size: 2,
          insert_spaces: true,
          properties: HashMap::new(),
          trim_trailing_whitespace: Some(true),
          insert_final_newline: Some(true),
          trim_final_newlines: Some(true),
        },
        work_done_progress_params: WorkDoneProgressParams { work_done_token: None },
      },
    )
    .await
    .context("formatting request after didClose failed")?;

  if formatting_after_close.error.is_none() {
    println!("⚠️  Warning: formatting after didClose succeeded; server may not have cleared document state");
  } else {
    println!("✅ Server correctly rejected formatting after didClose");
  }

  let invalid_shutdown_response = client
    .send_request_raw(Shutdown::METHOD, json!({ "unexpected": true }))
    .await
    .context("shutdown request with invalid params failed")?;

  let invalid_shutdown_error = invalid_shutdown_response
    .error
    .ok_or_else(|| anyhow::anyhow!("shutdown with invalid params unexpectedly succeeded"))?;

  if invalid_shutdown_error.code != -32602 {
    anyhow::bail!(
      "shutdown with invalid params returned unexpected error code {}; expected -32602",
      invalid_shutdown_error.code
    );
  }

  if !invalid_shutdown_error.message.contains("Invalid params") {
    anyhow::bail!(
      "shutdown with invalid params returned unexpected error message: {}",
      invalid_shutdown_error.message
    );
  }

  client
    .send_request::<Shutdown>(())
    .await
    .context("shutdown request failed")?;
  client
    .send_notification::<lsp_types::notification::Exit>(())
    .await
    .context("exit notification failed")?;
  client.shutdown_and_wait().await?;

  println!("✅ LSP E2E test passed");
  Ok(())
}

fn get_workspace_root() -> Result<PathBuf> {
  let output = Command::new("cargo")
    .args(["locate-project", "--workspace", "--message-format=plain"])
    .output()
    .context("Failed to locate workspace root")?;

  if !output.status.success() {
    anyhow::bail!("cargo locate-project failed");
  }

  let cargo_toml = String::from_utf8(output.stdout).context("Workspace path is not valid UTF-8")?;
  let cargo_toml = cargo_toml.trim();
  let parent = std::path::Path::new(cargo_toml)
    .parent()
    .context("Failed to get workspace root from Cargo.toml path")?;

  Ok(parent.to_path_buf())
}
