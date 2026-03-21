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
  sync::mpsc::{Receiver, channel},
  time::{Duration, sleep, timeout},
};
use tower_lsp::{
  jsonrpc::{self, ErrorCode},
  lsp_types::{
    ClientCapabilities, CompletionContext, CompletionParams, CompletionResponse, CompletionTriggerKind,
    DidChangeTextDocumentParams, DidChangeWatchedFilesClientCapabilities, DidChangeWatchedFilesParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentFormattingParams,
    DynamicRegistrationClientCapabilities, FileChangeType, FileEvent, FormattingOptions, Hover, HoverParams,
    HoverProviderCapability, InitializeParams, InitializeResult, OneOf, Position, PublishDiagnosticsParams, ServerInfo,
    TextDocumentClientCapabilities, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncClientCapabilities, TextDocumentSyncKind,
    TextEdit, Url, VersionedTextDocumentIdentifier, WorkDoneProgressParams, WorkspaceClientCapabilities,
    notification::{DidChangeTextDocument, DidChangeWatchedFiles, DidCloseTextDocument, DidOpenTextDocument},
    request::{
      Completion, Formatting, HoverRequest, RegisterCapability, Request as LspRequest, WorkDoneProgressCreate,
      WorkspaceConfiguration,
    },
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
    run_lsp_hover_e2e_test_impl(&binary).await?;
    run_lsp_completion_e2e_test_impl(&binary).await?;
    completion_returns_empty_for_trigger_character_requests(&binary).await?;
    completion_updates_after_watched_metadata_change(&binary).await?;
    completion_returns_empty_when_index_manager_missing(&binary).await?;
    run_lsp_metadata_completion_e2e_test_impl(&binary).await?;
    run_lsp_metadata_document_completion_e2e_test_impl(&binary).await?;
    hover_and_completion_do_not_panic_on_parse_error(&binary).await?;
    formatting_regression_stays_green_after_p3_changes(&binary).await?;
    run_lsp_diagnostics_e2e_test_impl(&binary).await?;
    run_lsp_diagnostics_clear_e2e_test_impl(&binary).await?;
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

  let (server, initialize_result, _, pid_file, _diagnostics_rx) = start_server(binary, workspace.path()).await?;
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

  let (server, _, watched_registration_ready, pid_file, _diagnostics_rx) =
    start_server(binary, workspace.path()).await?;
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

async fn run_lsp_hover_e2e_test_impl(binary: &Path) -> Result<()> {
  println!("• run_lsp_hover_e2e_test");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_path = workspace
    .path()
    .join("book")
    .join("metadata")
    .join("characters")
    .join("hero.md");
  std::fs::create_dir_all(metadata_path.parent().context("Missing metadata parent")?)
    .context("Failed to create metadata parent")?;

  let metadata_text = "---\ntitle: Hero Alpha\ntype: character\n---\nBrave hero body.";
  std::fs::write(&metadata_path, metadata_text).context("Failed to write metadata file")?;

  let (server, initialize_result, _, pid_file, _diagnostics_rx) = start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;

  let metadata_uri = file_url(&metadata_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: metadata_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: metadata_text.to_string(),
      },
    })
    .await;

  let hover = request_hover(&server, metadata_uri, Position { line: 1, character: 3 }).await?;
  let hover = hover.ok_or_else(|| anyhow!("hover request returned None for metadata document"))?;
  let rendered = hover_contents_to_string(&hover);
  assert!(
    rendered.contains("Hero Alpha"),
    "hover should include title/name, got: {rendered}"
  );
  assert!(
    rendered.contains("character"),
    "hover should include entity type, got: {rendered}"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ metadata hover passed");
  Ok(())
}

async fn run_lsp_completion_e2e_test_impl(binary: &Path) -> Result<()> {
  println!("• run_lsp_completion_e2e_test");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_dir = workspace.path().join("book").join("metadata").join("characters");
  let article_path = workspace.path().join("chapter-01.md");
  std::fs::create_dir_all(&metadata_dir).context("Failed to create metadata directory")?;

  std::fs::write(
    metadata_dir.join("hero-alpha.md"),
    "---\ntitle: Hero Alpha\ntype: character\n---\nAlpha body",
  )
  .context("Failed to write hero-alpha metadata")?;
  std::fs::write(
    metadata_dir.join("hero-beta.md"),
    "---\ntitle: Hero Beta\ntype: character\n---\nBeta body",
  )
  .context("Failed to write hero-beta metadata")?;
  std::fs::write(&article_path, "Hero").context("Failed to write article.md")?;

  let (server, initialize_result, watched_registration_ready, pid_file, _diagnostics_rx) =
    start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;
  wait_for_watched_registration(&watched_registration_ready, 5).await?;

  notify_watched_files(
    &server,
    vec![
      file_event(&metadata_dir.join("hero-alpha.md"), FileChangeType::CREATED)?,
      file_event(&metadata_dir.join("hero-beta.md"), FileChangeType::CREATED)?,
    ],
  )
  .await?;
  sleep(Duration::from_millis(200)).await;

  let article_uri = file_url(&article_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: article_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: "Hero".to_string(),
      },
    })
    .await;

  let completion = request_completion(&server, article_uri, Position { line: 0, character: 4 }).await?;
  let items = completion_items(completion).ok_or_else(|| anyhow!("completion request returned None"))?;
  assert!(
    items.iter().any(|item| {
      item.label == "Hero Alpha"
        && item.insert_text.as_deref() == Some("Hero Alpha")
        && item.detail.as_deref() == Some("character · book")
    }),
    "completion should include Hero Alpha with expected contract, got: {items:?}"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ manual completion passed");
  Ok(())
}

async fn run_lsp_diagnostics_e2e_test_impl(binary: &Path) -> Result<()> {
  println!("• run_lsp_diagnostics_e2e_test");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_path = workspace
    .path()
    .join("book\u{2f}metadata")
    .join("characters")
    .join("broken.md");
  std::fs::create_dir_all(metadata_path.parent().context("Missing metadata parent")?)
    .context("Failed to create metadata parent")?;

  let metadata_text = "---\ntitle: Broken Hero\nthis is not valid frontmatter\n---\nBody";
  std::fs::write(&metadata_path, metadata_text).context("Failed to write broken metadata")?;

  let (server, initialize_result, _, pid_file, mut diagnostics_rx) = start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;

  let metadata_uri = file_url(&metadata_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: metadata_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: metadata_text.to_string(),
      },
    })
    .await;

  let diagnostics = wait_for_diagnostics(&mut diagnostics_rx, &metadata_uri, 1, Duration::from_secs(2)).await?;
  assert_eq!(
    diagnostics.diagnostics.len(),
    1,
    "expected one diagnostic, got {diagnostics:?}"
  );
  assert_eq!(
    diagnostics.diagnostics[0].message,
    "Malformed frontmatter line: this is not valid frontmatter"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ diagnostics publish passed");
  Ok(())
}

async fn run_lsp_diagnostics_clear_e2e_test_impl(binary: &Path) -> Result<()> {
  println!("• run_lsp_diagnostics_clear_e2e_test");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_path = workspace
    .path()
    .join("book")
    .join("metadata")
    .join("characters")
    .join("recover.md");
  std::fs::create_dir_all(metadata_path.parent().context("Missing metadata parent")?)
    .context("Failed to create metadata parent")?;

  let broken_text = "---\ntitle: Recovering Hero\nthis is not valid frontmatter\n---\nBody";
  std::fs::write(&metadata_path, broken_text).context("Failed to write recover metadata")?;

  let (server, initialize_result, _, pid_file, mut diagnostics_rx) = start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;

  let metadata_uri = file_url(&metadata_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: metadata_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: broken_text.to_string(),
      },
    })
    .await;

  let first = wait_for_diagnostics(&mut diagnostics_rx, &metadata_uri, 1, Duration::from_secs(2)).await?;
  assert_eq!(
    first.diagnostics.len(),
    1,
    "expected one initial diagnostic, got {first:?}"
  );

  let fixed_text = "---\ntitle: Recovering Hero\ntype: character\n---\nBody";
  server
    .send_notification::<DidChangeTextDocument>(DidChangeTextDocumentParams {
      text_document: VersionedTextDocumentIdentifier {
        uri: metadata_uri.clone(),
        version: 2,
      },
      content_changes: vec![TextDocumentContentChangeEvent {
        range: None,
        range_length: None,
        text: fixed_text.to_string(),
      }],
    })
    .await;

  let cleared = wait_for_diagnostics(&mut diagnostics_rx, &metadata_uri, 2, Duration::from_secs(2)).await?;
  assert!(
    cleared.diagnostics.is_empty(),
    "expected diagnostics clear notification, got {cleared:?}"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ diagnostics clear passed");
  Ok(())
}

async fn completion_updates_after_watched_metadata_change(binary: &Path) -> Result<()> {
  println!("• completion_updates_after_watched_metadata_change");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_dir = workspace.path().join("book").join("metadata").join("characters");
  let metadata_path = metadata_dir.join("hero-refresh.md");
  let article_path = workspace.path().join("chapter-refresh.md");
  std::fs::create_dir_all(&metadata_dir).context("Failed to create metadata directory")?;

  std::fs::write(
    &metadata_path,
    "---\ntitle: Hero Before Refresh\ntype: character\n---\nBefore body",
  )
  .context("Failed to write initial metadata")?;
  std::fs::write(&article_path, "Hero").context("Failed to write chapter-refresh.md")?;

  let (server, initialize_result, watched_registration_ready, pid_file, _diagnostics_rx) =
    start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;
  wait_for_watched_registration(&watched_registration_ready, 5).await?;

  notify_watched_files(&server, vec![file_event(&metadata_path, FileChangeType::CREATED)?]).await?;
  sleep(Duration::from_millis(200)).await;

  let metadata_uri = file_url(&metadata_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: metadata_uri,
        language_id: "markdown".to_string(),
        version: 1,
        text: "---\ntitle: Hero Before Refresh\ntype: character\n---\nBefore body".to_string(),
      },
    })
    .await;

  std::fs::write(
    &metadata_path,
    "---\ntitle: Hero After Refresh\ntype: character\n---\nAfter body",
  )
  .context("Failed to update metadata")?;
  notify_watched_files(&server, vec![file_event(&metadata_path, FileChangeType::CHANGED)?]).await?;
  sleep(Duration::from_millis(250)).await;

  let article_uri = file_url(&article_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: article_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: "Hero".to_string(),
      },
    })
    .await;

  let completion = request_completion(&server, article_uri, Position { line: 0, character: 4 }).await?;
  let items = completion_items(completion).ok_or_else(|| anyhow!("completion request returned None"))?;

  assert!(
    items.iter().any(|item| item.label == "Hero After Refresh"),
    "completion should contain refreshed metadata title, got: {items:?}"
  );
  assert!(
    items.iter().all(|item| item.label != "Hero Before Refresh"),
    "completion should not contain stale metadata title, got: {items:?}"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ watched metadata refresh completion passed");
  Ok(())
}

async fn completion_returns_empty_for_trigger_character_requests(binary: &Path) -> Result<()> {
  println!("• completion_returns_empty_for_trigger_character_requests");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_dir = workspace.path().join("book").join("metadata").join("characters");
  let article_path = workspace.path().join("chapter-trigger.md");
  std::fs::create_dir_all(&metadata_dir).context("Failed to create metadata directory")?;

  std::fs::write(
    metadata_dir.join("hero-trigger.md"),
    "---\ntitle: Hero Trigger\ntype: character\n---\nTrigger body",
  )
  .context("Failed to write hero-trigger metadata")?;
  std::fs::write(&article_path, "Hero").context("Failed to write chapter-trigger.md")?;

  let (server, initialize_result, watched_registration_ready, pid_file, _diagnostics_rx) =
    start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;
  wait_for_watched_registration(&watched_registration_ready, 5).await?;

  notify_watched_files(
    &server,
    vec![file_event(
      &metadata_dir.join("hero-trigger.md"),
      FileChangeType::CREATED,
    )?],
  )
  .await?;
  sleep(Duration::from_millis(200)).await;

  let article_uri = file_url(&article_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: article_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: "Hero".to_string(),
      },
    })
    .await;

  let manual_completion = request_completion(&server, article_uri.clone(), Position { line: 0, character: 4 }).await?;
  let manual_items = completion_items(manual_completion).ok_or_else(|| anyhow!("completion request returned None"))?;
  assert!(
    !manual_items.is_empty(),
    "manual INVOKED completion should return items before trigger-character assertion"
  );

  let trigger_completion =
    request_completion_with_trigger_character(&server, article_uri, Position { line: 0, character: 4 }, "#").await?;
  let trigger_items =
    completion_items(trigger_completion).ok_or_else(|| anyhow!("completion request returned None"))?;
  assert!(
    trigger_items.is_empty(),
    "trigger-character completion should return empty list, got: {trigger_items:?}"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ trigger-character completion returns empty passed");
  Ok(())
}

async fn run_lsp_metadata_completion_e2e_test_impl(binary: &Path) -> Result<()> {
  println!("• run_lsp_metadata_completion_e2e_test");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_dir = workspace.path().join("book").join("metadata").join("characters");
  let article_path = workspace.path().join("chapter-01.md");
  std::fs::create_dir_all(&metadata_dir).context("Failed to create metadata directory")?;

  // Write metadata files BEFORE starting server
  std::fs::write(
    metadata_dir.join("hero-alpha.md"),
    "---\ntitle: Hero Alpha\ntype: character\n---\nAlpha body",
  )
  .context("Failed to write hero-alpha metadata")?;
  std::fs::write(
    metadata_dir.join("hero-beta.md"),
    "---\ntitle: Hero Beta\ntype: character\n---\nBeta body",
  )
  .context("Failed to write hero-beta metadata")?;
  std::fs::write(&article_path, "Hero").context("Failed to write article")?;

  let (server, initialize_result, watched_registration_ready, pid_file, _diagnostics_rx) =
    start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;
  wait_for_watched_registration(&watched_registration_ready, 5).await?;

  // Index both metadata files via watched file notifications
  notify_watched_files(
    &server,
    vec![
      file_event(&metadata_dir.join("hero-alpha.md"), FileChangeType::CREATED)?,
      file_event(&metadata_dir.join("hero-beta.md"), FileChangeType::CREATED)?,
    ],
  )
  .await?;
  sleep(Duration::from_millis(300)).await;

  // First open a metadata file
  let metadata_uri = file_url(&metadata_dir.join("hero-alpha.md"))?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: metadata_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: "---\ntitle: Hero Alpha\ntype: character\n---\nAlpha body".to_string(),
      },
    })
    .await;

  // Then open the article file and request completion there
  let article_uri = file_url(&article_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: article_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: "Hero".to_string(),
      },
    })
    .await;

  // Request completion in article document (proving metadata is indexed)
  let completion = request_completion(&server, article_uri, Position { line: 0, character: 4 }).await?;
  let items = completion_items(completion).ok_or_else(|| anyhow!("completion request returned None"))?;

  // Verify both metadata entities are available for completion
  assert!(
    items.iter().any(|item| item.label == "Hero Alpha"),
    "completion should include Hero Alpha from metadata, got: {items:?}"
  );
  assert!(
    items.iter().any(|item| item.label == "Hero Beta"),
    "completion should include Hero Beta from metadata, got: {items:?}"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ metadata completion e2e passed");
  Ok(())
}

async fn run_lsp_metadata_document_completion_e2e_test_impl(binary: &Path) -> Result<()> {
  println!("• run_lsp_metadata_document_completion_e2e_test");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_dir = workspace.path().join("book").join("metadata").join("characters");
  let hero_alpha_path = metadata_dir.join("hero-alpha.md");
  let hero_beta_path = metadata_dir.join("hero-beta.md");
  std::fs::create_dir_all(&metadata_dir).context("Failed to create metadata directory")?;

  let hero_alpha_text = "---\ntitle: Hero Alpha\ntype: character\n---\nHero";
  let hero_beta_text = "---\ntitle: Hero Beta\ntype: character\n---\nBeta body";
  std::fs::write(&hero_alpha_path, hero_alpha_text).context("Failed to write hero-alpha metadata")?;
  std::fs::write(&hero_beta_path, hero_beta_text).context("Failed to write hero-beta metadata")?;

  let (server, initialize_result, watched_registration_ready, pid_file, _diagnostics_rx) =
    start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;
  wait_for_watched_registration(&watched_registration_ready, 5).await?;

  notify_watched_files(
    &server,
    vec![
      file_event(&hero_alpha_path, FileChangeType::CREATED)?,
      file_event(&hero_beta_path, FileChangeType::CREATED)?,
    ],
  )
  .await?;
  sleep(Duration::from_millis(300)).await;

  let metadata_uri = file_url(&hero_alpha_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: metadata_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: hero_alpha_text.to_string(),
      },
    })
    .await;

  let completion = request_completion(&server, metadata_uri, Position { line: 4, character: 4 }).await?;
  let items = completion_items(completion).ok_or_else(|| anyhow!("completion request returned None"))?;
  assert!(
    items.iter().any(|item| {
      item.label == "Hero Alpha"
        && item.insert_text.as_deref() == Some("Hero Alpha")
        && item.detail.as_deref() == Some("character · book")
    }),
    "metadata-document completion should include Hero Alpha with expected shape, got: {items:?}"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ metadata-document completion e2e passed");
  Ok(())
}

async fn completion_returns_empty_when_index_manager_missing(binary: &Path) -> Result<()> {
  println!("• completion_returns_empty_when_index_manager_missing");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let mut init = initialize_params(workspace.path())?;
  init.root_uri = None;
  init.workspace_folders = None;

  let (server, initialize_result, _, pid_file, _diagnostics_rx) =
    start_server_with_initialize(binary, workspace.path(), init).await?;
  assert_core_capabilities(&initialize_result)?;

  let untitled_uri = Url::parse("untitled:chapter-01.md").context("Failed to build untitled URI")?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: untitled_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: "Hero".to_string(),
      },
    })
    .await;

  let completion = request_completion(&server, untitled_uri, Position { line: 0, character: 4 }).await?;
  let items = completion_items(completion).ok_or_else(|| anyhow!("completion request returned None"))?;
  assert!(
    items.is_empty(),
    "completion should gracefully return empty when index_manager is None, got: {items:?}"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ missing index_manager completion fallback passed");
  Ok(())
}

async fn hover_and_completion_do_not_panic_on_parse_error(binary: &Path) -> Result<()> {
  println!("• hover_and_completion_do_not_panic_on_parse_error");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let metadata_path = workspace
    .path()
    .join("book")
    .join("metadata")
    .join("characters")
    .join("parse-error.md");
  std::fs::create_dir_all(metadata_path.parent().context("Missing metadata parent")?)
    .context("Failed to create metadata parent")?;

  let broken_text = "---\ntitle: Parse Error Hero\ninvalid frontmatter line\n---\nBody";
  std::fs::write(&metadata_path, broken_text).context("Failed to write broken metadata")?;

  let (server, initialize_result, _, pid_file, _diagnostics_rx) = start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;

  let metadata_uri = file_url(&metadata_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: metadata_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: broken_text.to_string(),
      },
    })
    .await;

  let hover = request_hover(&server, metadata_uri.clone(), Position { line: 1, character: 3 }).await?;
  assert!(
    hover.is_some(),
    "hover should remain available for recoverable frontmatter issues"
  );

  let completion = request_completion(&server, metadata_uri, Position { line: 1, character: 3 }).await?;
  let items = completion_items(completion).ok_or_else(|| anyhow!("completion request returned None"))?;
  assert!(
    items.is_empty(),
    "completion should return empty on parse error document without panic, got: {items:?}"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ parse error hover/completion fallback passed");
  Ok(())
}

async fn formatting_regression_stays_green_after_p3_changes(binary: &Path) -> Result<()> {
  println!("• formatting_regression_stays_green_after_p3_changes");

  let workspace = TempDir::new().context("Failed to create temporary workspace")?;
  let article_path = workspace.path().join("formatting-regression.md");
  let article_text = "你好world\n\n第二段test";
  std::fs::write(&article_path, article_text).context("Failed to write formatting-regression.md")?;

  let (server, initialize_result, _, pid_file, _diagnostics_rx) = start_server(binary, workspace.path()).await?;
  assert_core_capabilities(&initialize_result)?;

  let article_uri = file_url(&article_path)?;
  server
    .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
      text_document: TextDocumentItem {
        uri: article_uri.clone(),
        language_id: "markdown".to_string(),
        version: 1,
        text: article_text.to_string(),
      },
    })
    .await;

  let edit = expect_single_edit(request_formatting(&server, article_uri).await?, "formatting regression")?;
  assert_eq!(
    edit.new_text, "    你好 world\n\n    第二段 test",
    "formatting behavior must remain stable after P3 changes"
  );

  shutdown_server(&server, pid_file.as_deref()).await?;
  drop(server);
  println!("  ✅ formatting regression remains green");
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
  workspace_root.join(".cache").join("novelsaga").join("sled")
}

async fn start_server(
  binary: &Path,
  workspace_root: &Path,
) -> Result<(
  LspServer,
  InitializeResult,
  Arc<AtomicBool>,
  Option<PathBuf>,
  Receiver<PublishDiagnosticsParams>,
)> {
  start_server_with_initialize(binary, workspace_root, initialize_params(workspace_root)?).await
}

async fn start_server_with_initialize(
  binary: &Path,
  workspace_root: &Path,
  params: InitializeParams,
) -> Result<(
  LspServer,
  InitializeResult,
  Arc<AtomicBool>,
  Option<PathBuf>,
  Receiver<PublishDiagnosticsParams>,
)> {
  let (program, args, pid_file) = lsp_launch_command(binary, workspace_root)?;
  let (server, rx) = LspServer::new(program, args);
  let watched_registration_ready = Arc::new(AtomicBool::new(false));
  let (diagnostics_tx, diagnostics_rx) = channel::<PublishDiagnosticsParams>(100);

  tokio::spawn(drive_server_messages(
    server.clone(),
    rx,
    watched_registration_ready.clone(),
    diagnostics_tx,
  ));

  let initialize_result = server
    .initialize(params)
    .await
    .map_err(|error| jsonrpc_error("initialize request failed", error))?;

  server.initialized().await;
  Ok((
    server,
    initialize_result,
    watched_registration_ready,
    pid_file,
    diagnostics_rx,
  ))
}

async fn drive_server_messages(
  server: LspServer,
  mut rx: Receiver<ServerMessage>,
  watched_registration_ready: Arc<AtomicBool>,
  diagnostics_tx: tokio::sync::mpsc::Sender<PublishDiagnosticsParams>,
) {
  while let Some(message) = rx.recv().await {
    match message {
      ServerMessage::Notification(notification) => {
        forward_diagnostics_notification(&notification.method, notification.params.as_ref(), &diagnostics_tx);
      }
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
    let helper_dir = workspace_root.join(".cache").join("novelsaga");
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

  // Task 6: hover and completion capabilities
  match result.capabilities.hover_provider {
    Some(HoverProviderCapability::Simple(true)) => {}
    ref other => bail!("expected hover provider enabled, got {other:?}"),
  }

  match &result.capabilities.completion_provider {
    Some(options) if options.trigger_characters.is_none() => {}
    ref other => bail!("expected completion provider with no trigger characters, got {other:?}"),
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

async fn request_hover(server: &LspServer, uri: Url, position: Position) -> Result<Option<Hover>> {
  server
    .send_request::<HoverRequest>(HoverParams {
      text_document_position_params: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri },
        position,
      },
      work_done_progress_params: WorkDoneProgressParams::default(),
    })
    .await
    .map_err(|error| jsonrpc_error("hover request failed", error))
}

async fn request_completion(server: &LspServer, uri: Url, position: Position) -> Result<Option<CompletionResponse>> {
  request_completion_with_context(
    server,
    uri,
    position,
    Some(CompletionContext {
      trigger_kind: CompletionTriggerKind::INVOKED,
      trigger_character: None,
    }),
  )
  .await
}

async fn request_completion_with_trigger_character(
  server: &LspServer,
  uri: Url,
  position: Position,
  trigger_character: &str,
) -> Result<Option<CompletionResponse>> {
  request_completion_with_context(
    server,
    uri,
    position,
    Some(CompletionContext {
      trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
      trigger_character: Some(trigger_character.to_string()),
    }),
  )
  .await
}

async fn request_completion_with_context(
  server: &LspServer,
  uri: Url,
  position: Position,
  context: Option<CompletionContext>,
) -> Result<Option<CompletionResponse>> {
  server
    .send_request::<Completion>(CompletionParams {
      text_document_position: TextDocumentPositionParams {
        text_document: TextDocumentIdentifier { uri },
        position,
      },
      context,
      work_done_progress_params: WorkDoneProgressParams::default(),
      partial_result_params: Default::default(),
    })
    .await
    .map_err(|error| jsonrpc_error("completion request failed", error))
}

async fn collect_diagnostics_notifications(
  diagnostics_rx: &mut Receiver<PublishDiagnosticsParams>,
  timeout_duration: Duration,
) -> Result<Vec<PublishDiagnosticsParams>> {
  let mut diagnostics = Vec::new();
  let deadline = tokio::time::Instant::now() + timeout_duration;

  while tokio::time::Instant::now() < deadline {
    match timeout(Duration::from_millis(100), diagnostics_rx.recv()).await {
      Ok(Some(diag)) => diagnostics.push(diag),
      Ok(None) => break,
      Err(_) => continue,
    }
  }

  Ok(diagnostics)
}

async fn wait_for_diagnostics(
  diagnostics_rx: &mut Receiver<PublishDiagnosticsParams>,
  uri: &Url,
  version: i32,
  timeout_duration: Duration,
) -> Result<PublishDiagnosticsParams> {
  let diagnostics = collect_diagnostics_notifications(diagnostics_rx, timeout_duration).await?;
  diagnostics
    .into_iter()
    .find(|params| params.uri == *uri && params.version == Some(version))
    .ok_or_else(|| anyhow!("timed out waiting for diagnostics for {uri} at version {version}"))
}

fn completion_items(response: Option<CompletionResponse>) -> Option<Vec<tower_lsp::lsp_types::CompletionItem>> {
  match response? {
    CompletionResponse::Array(items) => Some(items),
    CompletionResponse::List(list) => Some(list.items),
  }
}

fn hover_contents_to_string(hover: &Hover) -> String {
  match &hover.contents {
    tower_lsp::lsp_types::HoverContents::Scalar(marked) => marked_string_to_string(marked),
    tower_lsp::lsp_types::HoverContents::Array(items) => {
      items.iter().map(marked_string_to_string).collect::<Vec<_>>().join("\n")
    }
    tower_lsp::lsp_types::HoverContents::Markup(content) => content.value.clone(),
  }
}

fn marked_string_to_string(marked: &tower_lsp::lsp_types::MarkedString) -> String {
  match marked {
    tower_lsp::lsp_types::MarkedString::String(value) => value.clone(),
    tower_lsp::lsp_types::MarkedString::LanguageString(value) => value.value.clone(),
  }
}

/// Forwards a publishDiagnostics notification to the diagnostics channel.
/// Extracted as a helper for testability.
fn forward_diagnostics_notification(
  method: &str,
  params: Option<&Value>,
  diagnostics_tx: &tokio::sync::mpsc::Sender<PublishDiagnosticsParams>,
) {
  if method == "textDocument/publishDiagnostics" {
    if let Some(params) = params {
      if let Ok(diag_params) = serde_json::from_value::<PublishDiagnosticsParams>(params.clone()) {
        let _ = diagnostics_tx.try_send(diag_params);
      }
    }
  }
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

#[cfg(test)]
mod tests {
  use tokio::sync::mpsc::channel;

  use super::*;

  struct TestNotification {
    method: String,
    params: Option<Value>,
  }

  #[tokio::test]
  async fn test_diagnostics_forwarding() {
    let (diagnostics_tx, mut diagnostics_rx) = channel::<PublishDiagnosticsParams>(10);

    let notification = TestNotification {
      method: "textDocument/publishDiagnostics".to_string(),
      params: Some(serde_json::json!({
        "uri": "file:///test.md",
        "version": 1,
        "diagnostics": [{
          "range": {
            "start": {"line": 0, "character": 0},
            "end": {"line": 0, "character": 1}
          },
          "message": "test diagnostic"
        }]
      })),
    };

    forward_diagnostics_notification(&notification.method, notification.params.as_ref(), &diagnostics_tx);

    let received = timeout(Duration::from_secs(1), diagnostics_rx.recv()).await;
    assert!(received.is_ok());
    let params = received.unwrap().expect("should receive diagnostics");
    assert_eq!(params.uri.as_str(), "file:///test.md");
    assert_eq!(params.diagnostics.len(), 1);
    assert_eq!(params.diagnostics[0].message, "test diagnostic");
  }
}
