use std::{
  collections::HashMap,
  path::{Path, PathBuf},
  sync::Arc,
};

use novelsaga_core::{
  article::{Article, ArticleDocument},
  config::OverridableConfig,
  document::{DocumentKind, MarkdownParts, WorkspaceDocument},
  library,
  metadata::{
    MetadataEntity,
    parser::{generate_namespace, resolve_type},
  },
  state::init::Initializer,
};
use tokio::sync::RwLock;
use tower_lsp::{
  Client, LanguageServer,
  jsonrpc::Result as LspResult,
  lsp_types::{
    DeleteFilesParams, DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentFormattingParams, ExecuteCommandParams, FileChangeType, FileEvent,
    FileOperationFilter, FileOperationPattern, FileOperationPatternKind, FileOperationRegistrationOptions,
    FileSystemWatcher, InitializeParams, InitializeResult, InitializedParams, MessageType, OneOf, Position, Range,
    Registration, RenameFilesParams, ServerCapabilities, ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
    TextEdit, Url, WatchKind, WorkDoneProgressOptions, WorkspaceFileOperationsServerCapabilities,
    WorkspaceServerCapabilities,
  },
};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::metadata::{
  IndexManager,
  resolver::{MetadataResolver, ResolutionContext},
};

type DocumentStore = Arc<RwLock<HashMap<Url, DocumentState>>>;
type SharedIndexManager = Arc<RwLock<Option<Arc<IndexManager>>>>;

const WATCHED_METADATA_GLOB: &str = "**/metadata/**/*.md";
const WATCHED_CONFIG_GLOBS: [&str; 2] = ["**/novelsaga.config.*", "**/.novelsaga.*"];

#[derive(Debug, Clone)]
pub struct DocumentState {
  pub version: i32,
  pub text: Arc<str>,
  pub kind: DocumentKind,
  pub parsed: Result<WorkspaceDocument, String>,
  pub disk_changed: bool,
}

#[derive(Debug)]
pub struct Backend {
  client: Client,
  workspace_root: Arc<RwLock<Option<PathBuf>>>,
  document_store: DocumentStore,
  index_manager: SharedIndexManager,
  watched_files_dynamic_registration: Arc<RwLock<bool>>,
}

impl Backend {
  pub fn new(client: Client) -> Self {
    Self {
      client,
      workspace_root: Arc::new(RwLock::new(None)),
      document_store: Arc::new(RwLock::new(HashMap::new())),
      index_manager: Arc::new(RwLock::new(None)),
      watched_files_dynamic_registration: Arc::new(RwLock::new(false)),
    }
  }

  fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
  }

  fn document_path_from_url(uri: &Url) -> Result<PathBuf, String> {
    uri
      .to_file_path()
      .or_else(|()| uri.as_str().strip_prefix("file://").map(PathBuf::from).ok_or(()))
      .map_err(|()| format!("Failed to convert URI to file path: {uri}"))
  }

  fn classify_document(uri: &Url) -> DocumentKind {
    Self::document_path_from_url(uri).map_or_else(
      |_| DocumentKind::classify_path(Path::new(uri.path())),
      |path| DocumentKind::classify_path(&path),
    )
  }

  fn workspace_root_from_params(params: &InitializeParams) -> Option<PathBuf> {
    params
      .root_uri
      .as_ref()
      .or_else(|| {
        params
          .workspace_folders
          .as_ref()
          .and_then(|folders| folders.first().map(|folder| &folder.uri))
      })
      .and_then(|uri| Self::document_path_from_url(uri).ok())
  }

  fn workspace_root_from_file_uri(uri: &Url) -> Option<PathBuf> {
    Self::document_path_from_url(uri)
      .ok()
      .and_then(|path| path.parent().map(Path::to_path_buf))
  }

  fn workspace_watched_files_dynamic_registration(params: &InitializeParams) -> bool {
    params
      .capabilities
      .workspace
      .as_ref()
      .and_then(|workspace| workspace.did_change_watched_files.as_ref())
      .and_then(|capabilities| capabilities.dynamic_registration)
      .unwrap_or(false)
  }

  fn open_index_manager(workspace_root: Option<&Path>, lsp_startup_dir: Option<&Path>) -> Option<Arc<IndexManager>> {
    let context = ResolutionContext {
      workspace_root: workspace_root.map(std::path::Path::to_path_buf),
      cli_target_path: None,
      cli_cwd: None,
      show_target_parent: None,
      lsp_startup_dir: lsp_startup_dir.map(std::path::Path::to_path_buf),
    };

    let index_path = MetadataResolver::resolve(&context).ok()?;
    match IndexManager::open(&index_path) {
      Ok(manager) => Some(Arc::new(manager)),
      Err(error) => {
        eprintln!("Failed to open metadata index at {}: {error}", index_path.display());
        None
      }
    }
  }

  fn watched_globs() -> impl Iterator<Item = &'static str> {
    std::iter::once(WATCHED_METADATA_GLOB).chain(WATCHED_CONFIG_GLOBS)
  }

  fn watched_file_filters() -> Vec<FileOperationFilter> {
    Self::watched_globs()
      .map(|glob| FileOperationFilter {
        scheme: Some("file".to_string()),
        pattern: FileOperationPattern {
          glob: glob.to_string(),
          matches: Some(FileOperationPatternKind::File),
          options: None,
        },
      })
      .collect()
  }

  fn watched_file_operations_capabilities() -> WorkspaceFileOperationsServerCapabilities {
    let options = FileOperationRegistrationOptions {
      filters: Self::watched_file_filters(),
    };

    WorkspaceFileOperationsServerCapabilities {
      did_create: Some(options.clone()),
      will_create: None,
      did_rename: Some(options.clone()),
      will_rename: None,
      did_delete: Some(options),
      will_delete: None,
    }
  }

  fn watched_files_registration() -> Result<Registration, serde_json::Error> {
    let register_options = serde_json::to_value(tower_lsp::lsp_types::DidChangeWatchedFilesRegistrationOptions {
      watchers: Self::watched_globs()
        .map(|glob| FileSystemWatcher {
          glob_pattern: glob.to_string().into(),
          kind: Some(WatchKind::Create | WatchKind::Change | WatchKind::Delete),
        })
        .collect(),
    })?;

    Ok(Registration {
      id: format!("novelsaga-watched-files-{}", Uuid::new_v4()),
      method: "workspace/didChangeWatchedFiles".to_string(),
      register_options: Some(register_options),
    })
  }

  fn is_metadata_document(path: &Path) -> bool {
    DocumentKind::classify_path(path) == DocumentKind::Metadata
  }

  fn attach_entity_path(entity: &mut MetadataEntity, path: &Path) {
    let normalized_path = Self::normalize_path(path);

    if let Some(frontmatter) = entity.frontmatter.as_object_mut() {
      frontmatter.insert(
        "canonical_path".to_string(),
        serde_json::Value::String(normalized_path.to_string_lossy().into_owned()),
      );
      frontmatter.insert(
        "source_path".to_string(),
        serde_json::Value::String(path.to_string_lossy().into_owned()),
      );
    }
  }

  async fn index_manager(&self) -> Option<Arc<IndexManager>> {
    self.index_manager.read().await.clone()
  }

  async fn mark_document_disk_changed(&self, path: &Path) -> bool {
    let normalized_path = Self::normalize_path(path);
    let mut document_store = self.document_store.write().await;
    let mut matched = false;

    for (uri, state) in document_store.iter_mut() {
      let Ok(document_path) = Self::document_path_from_url(uri) else {
        continue;
      };

      if Self::normalize_path(&document_path) == normalized_path {
        state.disk_changed = true;
        matched = true;
      }
    }

    matched
  }

  async fn upsert_metadata_from_disk(&self, path: PathBuf) {
    if self.mark_document_disk_changed(&path).await {
      return;
    }

    let Some(index_manager) = self.index_manager().await else {
      return;
    };
    let workspace_root = { self.workspace_root.read().await.clone() };
    let Some(workspace_root) = workspace_root else {
      eprintln!("Skipping metadata refresh without workspace root: {}", path.display());
      return;
    };

    let content = match std::fs::read_to_string(&path) {
      Ok(content) => content,
      Err(error) => {
        eprintln!("Failed to read metadata file {}: {error}", path.display());
        return;
      }
    };

    let parts = MarkdownParts::parse(&content);
    let mut entity = match MetadataEntity::try_from((parts, path.as_path(), workspace_root.as_path())) {
      Ok(entity) => entity,
      Err(error) => {
        eprintln!("Failed to parse metadata file {}: {error}", path.display());
        return;
      }
    };

    Self::attach_entity_path(&mut entity, &path);
    if let Err(error) = index_manager.index_entity(&entity) {
      eprintln!("Failed to update metadata index for {}: {error}", path.display());
    }
  }

  async fn remove_metadata_by_path(&self, path: &Path) {
    if self.mark_document_disk_changed(path).await {
      return;
    }

    let Some(index_manager) = self.index_manager().await else {
      return;
    };

    let Some(entity_id) = index_manager.get_id_by_path(path) else {
      return;
    };

    if let Err(error) = index_manager.remove_entity(&entity_id) {
      eprintln!(
        "Failed to remove metadata entity {entity_id} for {}: {error}",
        path.display()
      );
    }
  }

  async fn handle_watched_path_create_or_change(&self, path: PathBuf) {
    if Self::is_metadata_document(&path) {
      self.upsert_metadata_from_disk(path).await;
    }
  }

  async fn handle_watched_path_delete(&self, path: PathBuf) {
    if Self::is_metadata_document(&path) {
      self.remove_metadata_by_path(&path).await;
    }
  }

  async fn handle_metadata_rename(&self, old_path: PathBuf, new_path: PathBuf) {
    let old_document_open = self.mark_document_disk_changed(&old_path).await;
    let new_document_open = self.mark_document_disk_changed(&new_path).await;

    if old_document_open || new_document_open {
      return;
    }

    self.remove_metadata_by_path(&old_path).await;
    self.upsert_metadata_from_disk(new_path).await;
  }

  async fn handle_file_change_event(&self, change: FileEvent) {
    let path = match Self::document_path_from_url(&change.uri) {
      Ok(path) => path,
      Err(error) => {
        self
          .client
          .log_message(
            MessageType::WARNING,
            format!("Ignoring watched file event for {}: {error}", change.uri),
          )
          .await;
        return;
      }
    };

    if !Self::is_metadata_document(&path) {
      return;
    }

    if change.typ == FileChangeType::DELETED {
      self.handle_watched_path_delete(path).await;
      return;
    }

    self.handle_watched_path_create_or_change(path).await;
  }

  fn path_from_uri_str(uri: &str) -> Result<PathBuf, String> {
    let parsed = Url::parse(uri).map_err(|error| format!("Invalid URI {uri}: {error}"))?;
    Self::document_path_from_url(&parsed)
  }

  fn parse_document(
    uri: &Url,
    text: &Arc<str>,
    workspace_root: Option<&Path>,
  ) -> (DocumentKind, Result<WorkspaceDocument, String>) {
    let kind = Self::classify_document(uri);

    let parsed = match Self::document_path_from_url(uri) {
      Ok(path) => {
        let parts = MarkdownParts::parse(text.as_ref());
        match kind {
          DocumentKind::Metadata => match workspace_root {
            Some(root) => MetadataEntity::try_from((parts, path.as_path(), root)).map(WorkspaceDocument::Metadata),
            None => Err(format!("Workspace root is required to parse metadata document: {uri}")),
          },
          DocumentKind::Article => Ok(WorkspaceDocument::Article(ArticleDocument::from_parts(parts))),
        }
      }
      Err(err) => Err(err),
    };

    (kind, parsed)
  }

  async fn refresh_document_parse(&self, uri: &Url) {
    let workspace_root = { self.workspace_root.read().await.clone() };
    let Some((version, text)) = ({
      let document_store = self.document_store.read().await;
      document_store
        .get(uri)
        .map(|state| (state.version, Arc::clone(&state.text)))
    }) else {
      return;
    };

    let (kind, parsed) = Self::parse_document(uri, &text, workspace_root.as_deref());

    let mut document_store = self.document_store.write().await;
    let Some(state) = document_store.get_mut(uri) else {
      return;
    };

    if state.version == version && Arc::ptr_eq(&state.text, &text) {
      state.kind = kind;
      state.parsed = parsed;
    }
  }
}

#[tower_lsp::async_trait]
#[allow(clippy::too_many_lines)]
impl LanguageServer for Backend {
  async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
    eprintln!("NovelSaga LSP Server initializing...");

    let workspace_root = Self::workspace_root_from_params(&params);
    if let Some(root_path) = workspace_root.as_ref() {
      eprintln!("Workspace root: {}", root_path.display());
    } else {
      eprintln!("Workspace root: <none>");
    }

    *self.workspace_root.write().await = workspace_root.clone();
    let lsp_startup_dir = workspace_root.as_deref();
    *self.index_manager.write().await =
      Self::open_index_manager(self.workspace_root.read().await.as_deref(), lsp_startup_dir);

    *self.watched_files_dynamic_registration.write().await =
      Self::workspace_watched_files_dynamic_registration(&params);

    Ok(InitializeResult {
      capabilities: ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        document_formatting_provider: Some(OneOf::Left(true)),
        execute_command_provider: Some(tower_lsp::lsp_types::ExecuteCommandOptions {
          commands: vec![
            "novelsaga/index".to_string(),
            "novelsaga/list".to_string(),
            "novelsaga/show".to_string(),
          ],
          work_done_progress_options: WorkDoneProgressOptions::default(),
        }),
        workspace: Some(WorkspaceServerCapabilities {
          workspace_folders: None,
          file_operations: Some(Self::watched_file_operations_capabilities()),
        }),
        ..Default::default()
      },
      server_info: Some(ServerInfo {
        name: "NovelSaga Language Server".to_string(),
        version: Some("0.1.0".to_string()),
      }),
    })
  }

  async fn initialized(&self, _: InitializedParams) {
    eprintln!("NovelSaga LSP Server initialized!");

    if *self.watched_files_dynamic_registration.read().await {
      match Self::watched_files_registration() {
        Ok(registration) => {
          if let Err(error) = self.client.register_capability(vec![registration]).await {
            self
              .client
              .log_message(
                MessageType::WARNING,
                format!("Failed to register watched files capability: {error}"),
              )
              .await;
          }
        }
        Err(error) => {
          self
            .client
            .log_message(
              MessageType::WARNING,
              format!("Failed to serialize watched files registration: {error}"),
            )
            .await;
        }
      }
    }

    self
      .client
      .log_message(MessageType::INFO, "NovelSaga server initialized")
      .await;
  }

  async fn shutdown(&self) -> LspResult<()> {
    eprintln!("NovelSaga LSP Server shutting down...");
    Ok(())
  }

  async fn did_open(&self, params: DidOpenTextDocumentParams) {
    let uri = params.text_document.uri;
    let version = params.text_document.version;
    let text: Arc<str> = Arc::from(params.text_document.text);
    let kind = Self::classify_document(&uri);

    eprintln!("Document opened: {uri}");

    // Fallback chain: root_uri → workspaceFolders[0] → first file derivation → None
    let should_initialize_workspace = {
      let current_root = self.workspace_root.read().await;
      current_root.is_none()
    };

    if should_initialize_workspace && let Some(derived_root) = Self::workspace_root_from_file_uri(&uri) {
      eprintln!(
        "Workspace root derived from first opened file: {}",
        derived_root.display()
      );
      *self.workspace_root.write().await = Some(derived_root.clone());
      // Initialize index_manager with newly derived workspace_root
      *self.index_manager.write().await =
        Self::open_index_manager(Some(derived_root.as_path()), Some(derived_root.as_path()));
    }

    {
      let mut document_store = self.document_store.write().await;
      document_store.insert(
        uri.clone(),
        DocumentState {
          version,
          text,
          kind,
          parsed: Err("Document parsing in progress".to_string()),
          disk_changed: false,
        },
      );
    }

    self.refresh_document_parse(&uri).await;

    self
      .client
      .log_message(MessageType::INFO, format!("Opened document: {uri}"))
      .await;
  }

  async fn did_change(&self, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;

    eprintln!("Document changed: {uri}");

    if let Some(change) = params.content_changes.into_iter().next() {
      let version = params.text_document.version;
      let text: Arc<str> = Arc::from(change.text);
      let kind = Self::classify_document(&uri);

      {
        let mut document_store = self.document_store.write().await;
        document_store
          .entry(uri.clone())
          .and_modify(|state| {
            state.version = version;
            state.text = Arc::clone(&text);
            state.kind = kind;
            state.parsed = Err("Document parsing in progress".to_string());
            state.disk_changed = false;
          })
          .or_insert_with(|| DocumentState {
            version,
            text: Arc::clone(&text),
            kind,
            parsed: Err("Document parsing in progress".to_string()),
            disk_changed: false,
          });
      }

      self.refresh_document_parse(&uri).await;
    }
  }

  async fn did_close(&self, params: DidCloseTextDocumentParams) {
    eprintln!("Document closed: {}", params.text_document.uri);

    let mut document_store = self.document_store.write().await;
    document_store.remove(&params.text_document.uri);
  }

  async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
    for change in params.changes {
      self.handle_file_change_event(change).await;
    }
  }

  async fn did_delete_files(&self, params: DeleteFilesParams) {
    for file in params.files {
      match Self::path_from_uri_str(&file.uri) {
        Ok(path) => {
          if Self::is_metadata_document(&path) {
            self.handle_watched_path_delete(path).await;
          }
        }
        Err(error) => {
          self
            .client
            .log_message(MessageType::WARNING, format!("Ignoring deleted file event: {error}"))
            .await;
        }
      }
    }
  }

  async fn did_rename_files(&self, params: RenameFilesParams) {
    for file in params.files {
      match (
        Self::path_from_uri_str(&file.old_uri),
        Self::path_from_uri_str(&file.new_uri),
      ) {
        (Ok(old_path), Ok(new_path)) => {
          if Self::is_metadata_document(&old_path) || Self::is_metadata_document(&new_path) {
            self.handle_metadata_rename(old_path, new_path).await;
          }
        }
        (Err(error), _) | (_, Err(error)) => {
          self
            .client
            .log_message(MessageType::WARNING, format!("Ignoring renamed file event: {error}"))
            .await;
        }
      }
    }
  }

  async fn formatting(&self, params: DocumentFormattingParams) -> LspResult<Option<Vec<TextEdit>>> {
    eprintln!("Formatting requested for {:?}", params.text_document.uri);

    // 使用全局配置加载器获取配置
    let (config, maybe_err) = {
      let state = match Initializer::get() {
        Ok(s) => s,
        Err(e) => {
          dbg!("Failed to get global state for formatting:", e);
          return Ok(None);
        }
      };
      let res = state
        .config_manager()
        .get_override_config(Path::new(params.text_document.uri.as_str()));
      match res {
        Ok(cfg) => (Some(cfg), None),
        Err(err) => (None, Some(err.to_string())),
      }
    };
    if let Some(err) = maybe_err {
      self
        .client
        .log_message(
          MessageType::WARNING,
          format!(
            "No configuration found for document: {}. Using default formatting. Error: {}",
            params.text_document.uri, err
          ),
        )
        .await;
    }
    dbg!("Loaded config for formatting:", &config);

    // 获取文档内容
    let document_store = self.document_store.read().await;
    let Some(content) = document_store
      .get(&params.text_document.uri)
      .map(|state| Arc::clone(&state.text))
    else {
      return Ok(None);
    };
    drop(document_store);

    let content = content.as_ref();

    // 使用 pangu 格式化文本(在中英文之间添加空格)
    let formatted = library::formatter::format_text(
      &Article::new(content),
      &config.as_ref().unwrap_or(&OverridableConfig::default()).fmt,
    );

    // 计算文档的结束位置
    let line_count = u32::try_from(content.lines().count()).unwrap_or(0);
    let last_line = content.lines().last().unwrap_or("");
    let last_char = u32::try_from(last_line.chars().count()).unwrap_or(0);

    // 返回替换整个文档的 TextEdit
    Ok(Some(vec![TextEdit {
      range: Range {
        start: Position { line: 0, character: 0 },
        end: Position {
          line: line_count.saturating_sub(1),
          character: last_char,
        },
      },
      new_text: formatted.content_ref().to_string(),
    }]))
  }

  async fn execute_command(&self, params: ExecuteCommandParams) -> LspResult<Option<serde_json::Value>> {
    eprintln!("Execute command: {}", params.command);

    let to_invalid_params = |message: String| tower_lsp::jsonrpc::Error::invalid_params(message);
    let to_internal_error = |message: String| tower_lsp::jsonrpc::Error {
      code: tower_lsp::jsonrpc::ErrorCode::InternalError,
      message: message.into(),
      data: None,
    };

    match params.command.as_str() {
      "novelsaga/index" => {
        self
          .client
          .log_message(MessageType::INFO, "Executing novelsaga/index command")
          .await;

        let arg = params
          .arguments
          .first()
          .ok_or_else(|| to_invalid_params("novelsaga/index requires arguments[0].path".to_string()))?;
        let path_str = arg
          .get("path")
          .and_then(serde_json::Value::as_str)
          .ok_or_else(|| to_invalid_params("novelsaga/index requires string arguments[0].path".to_string()))?;
        let path = PathBuf::from(path_str);

        if !path.exists() {
          return Err(to_invalid_params(format!(
            "Directory does not exist: {}",
            path.display()
          )));
        }
        if !path.is_dir() {
          return Err(to_invalid_params(format!(
            "Path is not a directory: {}",
            path.display()
          )));
        }

        let context = ResolutionContext {
          workspace_root: None,
          cli_target_path: Some(path.clone()),
          cli_cwd: None,
          show_target_parent: None,
          lsp_startup_dir: None,
        };
        let db_path = MetadataResolver::resolve(&context)
          .map_err(|error| to_internal_error(format!("Failed to resolve metadata database path for index: {error}")))?;
        let index_manager = IndexManager::open(&db_path).map_err(|error| {
          to_internal_error(format!(
            "Failed to open metadata index at {}: {error}",
            db_path.display()
          ))
        })?;

        let md_files: Vec<PathBuf> = WalkDir::new(&path)
          .into_iter()
          .filter_map(std::result::Result::ok)
          .filter(|e| e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext.eq_ignore_ascii_case("md")))
          .map(|e| e.path().to_path_buf())
          .collect();

        let mut indexed = 0usize;
        let mut failed = 0usize;

        for file_path in md_files {
          let file_content = match std::fs::read_to_string(&file_path) {
            Ok(file_content) => file_content,
            Err(error) => {
              eprintln!("Failed to read markdown file {}: {error}", file_path.display());
              failed += 1;
              continue;
            }
          };

          let (frontmatter, body) = if let Some(rest) = file_content.strip_prefix("---") {
            if let Some(end) = rest.find("---") {
              let body_str = &rest[end + 3..].trim_start();
              (serde_json::json!({}), body_str.to_string())
            } else {
              (serde_json::json!({}), file_content)
            }
          } else {
            (serde_json::json!({}), file_content)
          };

          let id = IndexManager::generate_id(&file_path.to_string_lossy());
          let type_ = resolve_type(&file_path, &frontmatter);
          let namespace = generate_namespace(&file_path, &path);
          let entity = MetadataEntity::new(&id, &type_, &namespace, frontmatter, &body);

          if let Err(error) = index_manager.index_entity(&entity) {
            eprintln!("Failed to index {}: {error}", file_path.display());
            failed += 1;
            continue;
          }

          indexed += 1;
        }

        index_manager.flush().map_err(|error| {
          to_internal_error(format!(
            "Failed to flush metadata index at {}: {error}",
            db_path.display()
          ))
        })?;

        Ok(Some(serde_json::json!({
          "status": "ok",
          "command": "novelsaga/index",
          "db_path": db_path,
          "indexed": indexed,
          "failed": failed
        })))
      }
      "novelsaga/list" => {
        self
          .client
          .log_message(MessageType::INFO, "Executing novelsaga/list command")
          .await;

        let workspace_root = { self.workspace_root.read().await.clone() };
        let context = ResolutionContext {
          workspace_root,
          cli_target_path: None,
          cli_cwd: None,
          show_target_parent: None,
          lsp_startup_dir: None,
        };
        let db_path = MetadataResolver::resolve(&context)
          .map_err(|error| to_internal_error(format!("Failed to resolve metadata database path for list: {error}")))?;
        let index_manager = IndexManager::open(&db_path).map_err(|error| {
          to_internal_error(format!(
            "Failed to open metadata index at {}: {error}",
            db_path.display()
          ))
        })?;
        let entities = index_manager.list_all().map_err(|error| {
          to_internal_error(format!(
            "Failed to list metadata entities from {}: {error}",
            db_path.display()
          ))
        })?;

        Ok(Some(serde_json::json!({
          "status": "ok",
          "command": "novelsaga/list",
          "db_path": db_path,
          "count": entities.len(),
          "entities": entities
        })))
      }
      "novelsaga/show" => {
        self
          .client
          .log_message(MessageType::INFO, "Executing novelsaga/show command")
          .await;

        let arg = params
          .arguments
          .first()
          .ok_or_else(|| to_invalid_params("novelsaga/show requires arguments[0].path".to_string()))?;
        let path_str = arg
          .get("path")
          .and_then(serde_json::Value::as_str)
          .ok_or_else(|| to_invalid_params("novelsaga/show requires string arguments[0].path".to_string()))?;
        let canonical_path = PathBuf::from(path_str)
          .canonicalize()
          .map_err(|error| to_invalid_params(format!("Failed to canonicalize show path '{path_str}': {error}")))?;
        let show_target_parent = canonical_path.parent().map(Path::to_path_buf).ok_or_else(|| {
          to_invalid_params(format!(
            "Failed to get parent directory for show path: {}",
            canonical_path.display()
          ))
        })?;

        let context = ResolutionContext {
          workspace_root: None,
          cli_target_path: None,
          cli_cwd: None,
          show_target_parent: Some(show_target_parent),
          lsp_startup_dir: None,
        };
        let db_path = MetadataResolver::resolve(&context)
          .map_err(|error| to_internal_error(format!("Failed to resolve metadata database path for show: {error}")))?;
        let index_manager = IndexManager::open(&db_path).map_err(|error| {
          to_internal_error(format!(
            "Failed to open metadata index at {}: {error}",
            db_path.display()
          ))
        })?;

        let entity_id = IndexManager::generate_id(&canonical_path.to_string_lossy());
        let entity = index_manager.get_by_id(&entity_id).map_err(|error| {
          to_internal_error(format!(
            "Failed to query metadata entity for {}: {error}",
            canonical_path.display()
          ))
        })?;

        Ok(Some(serde_json::json!({
          "status": "ok",
          "command": "novelsaga/show",
          "db_path": db_path,
          "entity_id": entity_id,
          "path": canonical_path,
          "entity": entity
        })))
      }
      _ => {
        self
          .client
          .log_message(MessageType::WARNING, format!("Unknown command: {}", params.command))
          .await;
        Ok(None)
      }
    }
  }
}
