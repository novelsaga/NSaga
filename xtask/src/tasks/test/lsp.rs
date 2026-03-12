use std::{
  path::{Path, PathBuf},
  process::{Command, Stdio},
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};

use anyhow::{Context, Result, anyhow, bail};
use async_lsp_client::{LspServer, ServerMessage};
use serde_json::Value;
use tempfile::TempDir;
use tokio::{
  runtime::Runtime,
  sync::mpsc::Receiver,
  time::{Duration, sleep},
};
use tower_lsp::{
  jsonrpc::{self, ErrorCode},
  lsp_types::{
    ClientCapabilities, DidChangeTextDocumentParams, DidChangeWatchedFilesClientCapabilities,
    DidChangeWatchedFilesParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
    DynamicRegistrationClientCapabilities, FileChangeType, FileEvent, FormattingOptions, InitializeParams,
    InitializeResult, OneOf, ServerInfo, TextDocumentClientCapabilities, TextDocumentContentChangeEvent,
    TextDocumentIdentifier, TextDocumentItem, TextDocumentSyncCapability, TextDocumentSyncClientCapabilities,
    TextDocumentSyncKind, TextEdit, Url, VersionedTextDocumentIdentifier, WorkDoneProgressParams,
    WorkspaceClientCapabilities,
    notification::{DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument},
    request::{Formatting, RegisterCapability, Request as LspRequest, WorkDoneProgressCreate, WorkspaceConfiguration},
  },
};

#[derive(serde::Deserialize, serde::Serialize)]
struct ShutdownParams {}

enum ShutdownRequest {}

impl LspRequest for ShutdownRequest {
  type Params = ShutdownParams;
  type Result = ();

  const METHOD: &'static str = "shutdown";
}

#[cfg(not(windows))]
use std::os::unix::fs::PermissionsExt;

use crate::tasks::utils::{project_root, run_command, target_dir};

#[cfg(windows)]
const NOVELSAGA_BINARY_NAME: &str = "novelsaga.exe";
#[cfg(not(windows))]
const NOVELSAGA_BINARY_NAME: &str = "novelsaga";

pub fn run_e2e_test() -> Result<()> {
  ensure_lsp_test_prerequisites()?;
  let binary = novelsaga_binary_path()?;
  let runtime = build_runtime()?;

  println!("🧪 Running LSP E2E tests...\n");

  runtime.block_on(async {
    run_lsp_e2e_test_impl(&binary).await?;
    run_lsp_didchangewatchedfiles_e2e_test_impl(&binary).await?;
    Ok::<(), anyhow::Error>(())
  })?;

  println!("\n✅ All LSP E2E tests passed");
  Ok(())
}

async fn run_lsp_e2e_test_impl(binary: &Path) -> Result<()> {
  println!("• run_lsp_e2e_test");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let article_path = workspace.path().join("article.md");
  let original_text = "你好world\n\n第二段test";
  let changed_text = "修改后abc\n\n第三段xyz";

  std::fs::write(&article_path, original_text).context("Failed to write initial article.md")?;

  let (server, initialize_result, _, pid_file) = start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;

  let article_uri = file_url(&article_path)?;

  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: article_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: original_text.to_string(),
      },
    })
    .await;

  let open_edit = expect_single_edit(
    request_formatting(&server, article_uri.clone()).await?,
    "formatting after didOpen",
  )?;
  assert_eq!(
    open_edit.new_text, "    你好 world\n\n    第二段 test",
    "didOpen formatting should use opened document contents"
  );

  server
    .send_notification::<DidChangeTextDocument>(DidChangeTextDocumentParams {
      text_document: VersionedTextDocumentIdentifier {
        uri: article_uri.clone(),
        version: 2,
      },
      content_changes: vec![TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: changed_text.to_string(),
      }],
    })
    .await;

  let changed_edit = expect_single_edit(
    request_formatting(&server, article_uri.clone()).await?,
    "formatting after didChange",
  )?;
  assert_eq!(
    changed_edit.new_text, "    修改后 abc\n\n    第三段 xyz",
    "didChange formatting should reflect updated in-memory contents"
  );

  server
    .send_notification::<DidCloseTextDocument>(DidCloseTextDocumentParams {
      text_document: TextDocumentIdentifier {
        uri: article_uri.clone(),
      },
    })
    .await;

  let after_close = request_formatting(&server, article_uri).await?;
  if after_close.is_some() {
    bail!("formatting after didClose should return None");
  }

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ core LSP operations passed");
  Ok(())
}

async fn run_lsp_didchangewatchedfiles_e2e_test_impl(binary: &Path) -> Result<()> {
  println!("• run_lsp_didchangewatchedfiles_e2e_test");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_dir = workspace.path().join("book").join("metadata");
  std::fs::create_dir_all(&metadata_dir).context("Failed to create metadata directory")?;

  let watched1 = metadata_dir.join("characters").join("watched1.md");
  let watched2 = metadata_dir.join("characters").join("watched2.md");
  let watched3 = metadata_dir.join("characters").join("watched3.md");
  std::fs::create_dir_all(watched1.parent().context("Missing watched parent")?)
    .context("Failed to create watched metadata parent directory")?;
  let index_dir = expected_index_dir_for_workspace(workspace.path());

  let (server, _, watched_registration_ready, pid_file) = start_server(binary, workspace.path()).await?;
  wait_for_watched_registration(&watched_registration_ready, 5).await?;

  std::fs::write(&watched1, "---\ntitle: Watched File 1\n---\n\nbody").context("Failed to write watched1.md")?;
  notify_watched_files(&server, vec![file_event(&watched1, FileChangeType::CREATED)?]).await?;
  sleep(Duration::from_millis(150)).await;

  std::fs::write(&watched1, "---\ntitle: Watched File 1 Updated\n---\n\nbody")
    .context("Failed to update watched1.md")?;
  notify_watched_files(&server, vec![file_event(&watched1, FileChangeType::CHANGED)?]).await?;
  sleep(Duration::from_millis(150)).await;

  std::fs::write(&watched2, "---\ntitle: Watched File 2\n---\n\nbody").context("Failed to write watched2.md")?;
  std::fs::write(&watched3, "---\ntitle: Watched File 3\n---\n\nbody").context("Failed to write watched3.md")?;
  notify_watched_files(
    &server,
    vec![
      file_event(&watched2, FileChangeType::CREATED)?,
      file_event(&watched3, FileChangeType::CREATED)?,
    ],
  )
  .await?;
  sleep(Duration::from_millis(150)).await;

  std::fs::remove_file(&watched3).context("Failed to delete watched3.md")?;
  notify_watched_files(&server, vec![file_event(&watched3, FileChangeType::DELETED)?]).await?;
  sleep(Duration::from_millis(150)).await;

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);

  wait_for_path(&index_dir).await?;
  if !wait_for_index_state(
    &index_dir,
    |db| {
      let Some(bytes1) = db.get(b"entity:watched1")? else {
        return Ok(false);
      };
      let entity1: Value = serde_json::from_slice(&bytes1).context("Failed to deserialize watched1 entity JSON")?;
      let watched1_updated = entity1
        .get("frontmatter")
        .and_then(|frontmatter| frontmatter.get("title"))
        .and_then(Value::as_str)
        == Some("Watched File 1 Updated");

      let watched2_exists = db.get(b"entity:watched2")?.is_some();
      let watched3_removed = db.get(b"entity:watched3")?.is_none();

      Ok(watched1_updated && watched2_exists && watched3_removed)
    },
    5,
  )
  .await?
  {
    let snapshot =
      read_index_snapshot(&index_dir).unwrap_or_else(|error| format!("<failed to read index snapshot: {error:#}>"));
    bail!("Timed out waiting for watched files index state after notifications. Snapshot: {snapshot}");
  }

  assert_index_title(&index_dir, "watched1", "Watched File 1 Updated")?;
  assert_index_title(&index_dir, "watched2", "Watched File 2")?;

  if read_index_entity(&index_dir, "watched3")?.is_some() {
    bail!("watched3 should have been removed from metadata index after delete event");
  }

  println!("  ✅ watched files protocol passed");
  Ok(())
}

fn build_runtime() -> Result<Runtime> {
  tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .context("Failed to create tokio runtime for LSP E2E tests")
}

fn ensure_lsp_test_prerequisites() -> Result<()> {
  println!("🔧 Building JS bridge assets for LSP E2E...");
  crate::tasks::build::build_all()?;

  println!("🔨 Building novelsaga-cli...");
  let mut command = Command::new("cargo");
  command
    .args(["build", "-p", "novelsaga-cli"])
    .current_dir(project_root());
  run_command(&mut command).context("Failed to build novelsaga-cli for LSP E2E tests")
}

fn novelsaga_binary_path() -> Result<PathBuf> {
  let binary = target_dir().join("debug").join(NOVELSAGA_BINARY_NAME);
  if !binary.exists() {
    bail!("novelsaga binary not found at {}", binary.display());
  }
  Ok(binary)
}

fn expected_index_dir_for_workspace(workspace_root: &Path) -> PathBuf {
  workspace_root.join(".novelsaga").join("cache").join("index")
}

async fn start_server(
  binary: &Path,
  workspace_root: &Path,
) -> Result<(LspServer, InitializeResult, Arc<AtomicBool>, Option<PathBuf>)> {
  let (program, args, pid_file) = lsp_launch_command(binary, workspace_root)?;
  let (server, rx) = LspServer::new(program, args);
  let watched_registration_ready = Arc::new(AtomicBool::new(false));

  tokio::spawn(drive_server_messages(
    server.clone(),
    rx,
    watched_registration_ready.clone(),
  ));

  let initialize_result = server
    .initialize(initialize_params(workspace_root)?)
    .await
    .map_err(|error| jsonrpc_error("initialize request failed", error))?;

  server.initialized().await;
  Ok((server, initialize_result, watched_registration_ready, pid_file))
}

async fn drive_server_messages(
  server: LspServer,
  mut rx: Receiver<ServerMessage>,
  watched_registration_ready: Arc<AtomicBool>,
) {
  while let Some(message) = rx.recv().await {
    match message {
      ServerMessage::Notification(_) => {}
      ServerMessage::Request(request) => {
        let Some(id) = request.id().cloned() else {
          continue;
        };

        match request.method() {
          <RegisterCapability as LspRequest>::METHOD => {
            watched_registration_ready.store(true, Ordering::Release);
            server.send_response::<RegisterCapability>(id, ()).await;
          }
          <WorkspaceConfiguration as LspRequest>::METHOD => {
            server
              .send_response::<WorkspaceConfiguration>(id, Vec::<Value>::new())
              .await;
          }
          <WorkDoneProgressCreate as LspRequest>::METHOD => {
            server.send_response::<WorkDoneProgressCreate>(id, ()).await;
          }
          _ => {
            server
              .send_error_response(
                id,
                jsonrpc::Error {
                  code: ErrorCode::MethodNotFound,
                  message: "Method Not Found".into(),
                  data: request.params().cloned(),
                },
              )
              .await;
          }
        }
      }
    }
  }
}

async fn wait_for_watched_registration(flag: &AtomicBool, timeout_secs: u64) -> Result<()> {
  let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);
  while std::time::Instant::now() < deadline {
    if flag.load(Ordering::Acquire) {
      return Ok(());
    }
    sleep(Duration::from_millis(50)).await;
  }

  bail!("Timed out waiting for watched-files dynamic registration")
}

fn lsp_launch_command(binary: &Path, workspace_root: &Path) -> Result<(String, Vec<String>, Option<PathBuf>)> {
  #[cfg(not(windows))]
  {
    let helper_dir = workspace_root.join(".novelsaga");
    std::fs::create_dir_all(&helper_dir).context("Failed to create helper directory for LSP launch")?;

    let pid_file = helper_dir.join("lsp-e2e.pid");
    let wrapper = helper_dir.join("lsp-e2e-launch.sh");
    let script = format!(
      "#!/usr/bin/env bash\nset -euo pipefail\necho $$ > \"{}\"\nexec \"{}\" lsp\n",
      pid_file.display(),
      binary.display()
    );
    std::fs::write(&wrapper, script).context("Failed to write LSP launch wrapper")?;

    let mut permissions = std::fs::metadata(&wrapper)
      .context("Failed to stat LSP launch wrapper")?
      .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&wrapper, permissions).context("Failed to chmod LSP launch wrapper")?;

    let program = wrapper.to_string_lossy().into_owned();
    Ok((program, Vec::new(), Some(pid_file)))
  }

  #[cfg(windows)]
  {
    let program = binary.to_string_lossy().into_owned();
    Ok((program, vec!["lsp".to_string()], None))
  }
}

#[allow(deprecated)]
fn initialize_params(workspace_root: &Path) -> Result<InitializeParams> {
  Ok(InitializeParams {
    process_id: Some(std::process::id()),
    root_path: None,
    root_uri: Some(file_url(workspace_root)?),
    initialization_options: None,
    capabilities: client_capabilities(),
    trace: None,
    workspace_folders: None,
    client_info: None,
    locale: None,
  })
}

fn client_capabilities() -> ClientCapabilities {
  ClientCapabilities {
    workspace: Some(WorkspaceClientCapabilities {
      configuration: Some(true),
      did_change_watched_files: Some(DidChangeWatchedFilesClientCapabilities {
        dynamic_registration: Some(true),
        relative_pattern_support: Some(true),
      }),
      ..Default::default()
    }),
    text_document: Some(TextDocumentClientCapabilities {
      synchronization: Some(TextDocumentSyncClientCapabilities {
        dynamic_registration: Some(true),
        ..Default::default()
      }),
      formatting: Some(DynamicRegistrationClientCapabilities {
        dynamic_registration: Some(true),
      }),
      ..Default::default()
    }),
    ..Default::default()
  }
}

fn assert_core_capabilities(result: &InitializeResult) -> Result<()> {
  match result.capabilities.text_document_sync {
    Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)) => {}
    ref other => bail!("expected FULL textDocumentSync, got {other:?}"),
  }

  match result.capabilities.document_formatting_provider {
    Some(OneOf::Left(true)) => {}
    ref other => bail!("expected document formatting provider enabled, got {other:?}"),
  }

  match &result.server_info {
    Some(ServerInfo { name, .. }) if name.contains("NovelSaga") => {}
    other => bail!("expected NovelSaga server_info, got {other:?}"),
  }

  Ok(())
}

async fn request_formatting(server: &LspServer, uri: Url) -> Result<Option<Vec<TextEdit>>> {
  server
    .send_request::<Formatting>(DocumentFormattingParams {
      text_document: TextDocumentIdentifier { uri },
      options: FormattingOptions {
        tab_size: 2,
        insert_spaces: true,
        properties: Default::default(),
        trim_trailing_whitespace: Some(true),
        insert_final_newline: Some(true),
        trim_final_newlines: Some(true),
      },
      work_done_progress_params: WorkDoneProgressParams::default(),
    })
    .await
    .map_err(|error| jsonrpc_error("formatting request failed", error))
}

fn expect_single_edit(edits: Option<Vec<TextEdit>>, context: &str) -> Result<TextEdit> {
  let mut edits = edits.ok_or_else(|| anyhow!("{context}: server returned no edits"))?;
  if edits.len() != 1 {
    bail!("{context}: expected exactly one TextEdit, got {}", edits.len());
  }
  Ok(edits.remove(0))
}

async fn notify_watched_files(server: &LspServer, changes: Vec<FileEvent>) -> Result<()> {
  server
    .send_notification::<DidChangeWatchedFiles>(DidChangeWatchedFilesParams { changes })
    .await;
  Ok(())
}

fn file_event(path: &Path, typ: FileChangeType) -> Result<FileEvent> {
  Ok(FileEvent::new(file_url(path)?, typ))
}

async fn shutdown_server(server: &LspServer, pid_file: Option<&Path>) -> Result<()> {
  let shutdown_rejected = match server.send_request::<ShutdownRequest>(ShutdownParams {}).await {
    Ok(()) => false,
    Err(error) if error.code == ErrorCode::InvalidParams => true,
    Err(_) => true,
  };

  if shutdown_rejected {
    let _ = server.shutdown().await;
  }

  server.exit().await;
  sleep(Duration::from_millis(200)).await;

  if let Some(pid_file) = pid_file {
    wait_for_lsp_process_exit(pid_file)?;
  }

  Ok(())
}

fn wait_for_lsp_process_exit(pid_file: &Path) -> Result<()> {
  if !pid_file.exists() {
    return Ok(());
  }

  #[cfg(not(windows))]
  {
    let pid = std::fs::read_to_string(pid_file)
      .context("Failed to read LSP pid file")?
      .trim()
      .parse::<i32>()
      .context("Failed to parse LSP pid")?;

    for _ in 0..20 {
      if !is_process_alive(pid) {
        let _ = std::fs::remove_file(pid_file);
        return Ok(());
      }
      std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let _ = Command::new("kill")
      .args(["-TERM", &pid.to_string()])
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();

    for _ in 0..20 {
      if !is_process_alive(pid) {
        let _ = std::fs::remove_file(pid_file);
        return Ok(());
      }
      std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let _ = Command::new("kill")
      .args(["-KILL", &pid.to_string()])
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();

    for _ in 0..20 {
      if !is_process_alive(pid) {
        let _ = std::fs::remove_file(pid_file);
        return Ok(());
      }
      std::thread::sleep(std::time::Duration::from_millis(100));
    }

    bail!("LSP process {pid} did not exit after shutdown/exit/TERM/KILL")
  }

  #[cfg(windows)]
  {
    let _ = std::fs::remove_file(pid_file);
    Ok(())
  }
}

#[cfg(not(windows))]
fn is_process_alive(pid: i32) -> bool {
  #[cfg(target_os = "linux")]
  {
    if is_process_zombie(pid) {
      return false;
    }
  }

  Command::new("kill")
    .args(["-0", &pid.to_string()])
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()
    .map(|status| status.success())
    .unwrap_or(false)
}

#[cfg(all(not(windows), target_os = "linux"))]
fn is_process_zombie(pid: i32) -> bool {
  let stat_path = format!("/proc/{pid}/stat");
  let Ok(stat) = std::fs::read_to_string(stat_path) else {
    return false;
  };

  stat
    .split_whitespace()
    .nth(2)
    .map(|state| state == "Z")
    .unwrap_or(false)
}

fn read_index_entity(index_dir: &Path, entity_id: &str) -> Result<Option<Value>> {
  let db = open_index_db_retry(index_dir, 50)?;
  let key = format!("entity:{entity_id}");
  let value = db
    .get(key.as_bytes())
    .with_context(|| format!("Failed to read entity {entity_id} from sled"))?;

  value
    .map(|bytes| serde_json::from_slice::<Value>(&bytes).context("Failed to deserialize metadata entity JSON"))
    .transpose()
}

fn assert_index_title(index_dir: &Path, entity_id: &str, expected_title: &str) -> Result<()> {
  let entity = read_index_entity(index_dir, entity_id)?
    .ok_or_else(|| anyhow!("Indexed entity {entity_id} was not found in {}", index_dir.display()))?;

  let actual_title = entity
    .get("frontmatter")
    .and_then(|frontmatter| frontmatter.get("title"))
    .and_then(Value::as_str)
    .ok_or_else(|| anyhow!("Entity {entity_id} does not contain frontmatter.title"))?;

  if actual_title != expected_title {
    bail!("Entity {entity_id} title mismatch: expected {expected_title:?}, got {actual_title:?}");
  }

  Ok(())
}

fn read_index_snapshot(index_dir: &Path) -> Result<String> {
  let db = open_index_db_retry(index_dir, 5)?;
  let mut entries = Vec::new();
  for item in db.scan_prefix(b"entity:") {
    let (key, value) = item.context("Failed to scan entity entries")?;
    let key = String::from_utf8(key.to_vec()).context("Invalid UTF-8 entity key")?;
    let entity: Value = serde_json::from_slice(&value).context("Failed to deserialize entity in snapshot")?;
    let title = entity
      .get("frontmatter")
      .and_then(|frontmatter| frontmatter.get("title"))
      .and_then(Value::as_str)
      .unwrap_or("<none>");
    entries.push(format!("{key}={title}"));
  }

  if entries.is_empty() {
    Ok("<empty>".to_string())
  } else {
    entries.sort();
    Ok(entries.join(", "))
  }
}

async fn wait_for_path(path: &Path) -> Result<()> {
  for _ in 0..20 {
    if path.exists() {
      return Ok(());
    }
    sleep(Duration::from_millis(100)).await;
  }

  bail!("Timed out waiting for path {}", path.display())
}

async fn wait_for_index_state<F>(index_dir: &Path, predicate: F, timeout_secs: u64) -> Result<bool>
where
  F: Fn(&sled::Db) -> Result<bool>,
{
  let deadline = std::time::Instant::now() + Duration::from_secs(timeout_secs);

  loop {
    let db = match open_index_db_retry(index_dir, 1) {
      Ok(db) => db,
      Err(_) => {
        if std::time::Instant::now() >= deadline {
          return Ok(false);
        }
        sleep(Duration::from_millis(100)).await;
        continue;
      }
    };
    if predicate(&db)? {
      return Ok(true);
    }

    if std::time::Instant::now() >= deadline {
      return Ok(false);
    }

    sleep(Duration::from_millis(100)).await;
  }
}

fn open_index_db_retry(index_dir: &Path, max_retries: usize) -> Result<sled::Db> {
  let mut attempt = 0usize;
  loop {
    match sled::open(index_dir) {
      Ok(db) => return Ok(db),
      Err(sled::Error::Io(error)) if error.kind() == std::io::ErrorKind::WouldBlock && attempt < max_retries => {
        attempt += 1;
        std::thread::sleep(std::time::Duration::from_millis(100));
      }
      Err(error) => {
        return Err(error).with_context(|| format!("Failed to open sled index at {}", index_dir.display()));
      }
    }
  }
}

fn file_url(path: &Path) -> Result<Url> {
  Url::from_file_path(path).map_err(|_| anyhow!("Failed to convert path to file URL: {}", path.display()))
}

fn jsonrpc_error(context: &str, error: jsonrpc::Error) -> anyhow::Error {
  anyhow!(
    "{context}: code={:?}, message={}, data={:?}",
    error.code,
    error.message,
    error.data
  )
}
