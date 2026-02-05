---
name: lsp-dev
description: LSP protocol implementation for NovelSaga - use when modifying LSP backend, document handlers, or language server capabilities
---

# NovelSaga LSP Development

## Quick Reference

- **Backend location**: `projects/cli/src/lsp/backend.rs`
- **Entry point**: `projects/cli/src/lsp/mod.rs` - `start()` function
- **Framework**: `tower_lsp` with async trait
- **Client communication**: `Client` handle for notifications/messages
- **Document storage**: `Arc<RwLock<HashMap<Url, String>>>`

```rust
// Basic backend structure
#[derive(Debug)]
pub struct Backend {
  client: Client,
  documents: Arc<RwLock<HashMap<Url, String>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> { ... }
  async fn initialized(&self, _: InitializedParams) { ... }
  async fn shutdown(&self) -> Result<()> { ... }
}
```

## Backend Trait

Implement `tower_lsp::LanguageServer` for your backend:

```rust
#[tower_lsp::async_trait]
impl LanguageServer for Backend {
  async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
    Ok(InitializeResult {
      capabilities: ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        document_formatting_provider: Some(OneOf::Left(true)),
        ..Default::default()
      },
      server_info: Some(ServerInfo {
        name: "NovelSaga Language Server".to_string(),
        version: Some("0.1.0".to_string()),
      }),
    })
  }

  async fn initialized(&self, _: InitializedParams) {
    self.client.log_message(MessageType::INFO, "Ready").await;
  }

  async fn shutdown(&self) -> Result<()> {
    Ok(())
  }
}
```

### Server Initialization

```rust
// lsp/mod.rs - creates service and starts server
pub async fn start() {
  let stdin = tokio::io::stdin();
  let stdout = tokio::io::stdout();

  let (service, socket) = LspService::new(Backend::new);
  tower_lsp::Server::new(stdin, stdout, socket).serve(service).await;
}
```

## Capabilities

Register capabilities during `initialize()`:

| Capability                     | Type                         | Example                  |
| ------------------------------ | ---------------------------- | ------------------------ |
| `text_document_sync`           | `TextDocumentSyncCapability` | Full or incremental sync |
| `document_formatting_provider` | `OneOf<bool, ...>`           | Enable format on save    |
| `completion_provider`          | `CompletionOptions`          | Auto-complete            |
| `hover_provider`               | `OneOf<bool, ...>`           | Hover info               |
| `diagnostic_provider`          | `DiagnosticServerOptions`    | Diagnostics              |

```rust
ServerCapabilities {
  // FULL: Server receives entire document on every change
  // INCREMENTAL: Server receives only changed ranges
  text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),

  // Enable formatting capability
  document_formatting_provider: Some(OneOf::Left(true)),

  ..Default::default()
}
```

## Document Sync

### FULL Sync Mode

Server receives **entire document content** on every change:

```rust
async fn did_change(&self, params: DidChangeTextDocumentParams) {
  // With FULL sync, first change contains complete document
  if let Some(change) = params.content_changes.into_iter().next() {
    let mut documents = self.documents.write().await;
    documents.insert(params.text_document.uri, change.text);
  }
}
```

**Use when**: Documents are small, formatting requires full context, or change tracking is complex.

### INCREMENTAL Sync Mode

Server receives only **changed ranges**:

```rust
async fn did_change(&self, params: DidChangeTextDocumentParams) {
  let mut documents = self.documents.write().await;

  if let Some(content) = documents.get_mut(&params.text_document.uri) {
    for change in params.content_changes {
      if let Some(range) = change.range {
        // Apply incremental change
        apply_edit(content, range, change.text);
      } else {
        // Full content replacement (can still happen)
        *content = change.text;
      }
    }
  }
}
```

**Use when**: Documents are large, performance is critical, or changes are frequent.

### Document Lifecycle

```rust
// Store document on open
async fn did_open(&self, params: DidOpenTextDocumentParams) {
  let mut documents = self.documents.write().await;
  documents.insert(params.text_document.uri.clone(), params.text_document.text);
}

// Clean up on close
async fn did_close(&self, params: DidCloseTextDocumentParams) {
  let mut documents = self.documents.write().await;
  documents.remove(&params.text_document.uri);
}
```

## Formatting Implementation

```rust
async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
  let documents = self.documents.read().await;
  let Some(content) = documents.get(&params.text_document.uri) else {
    return Ok(None);
  };

  // Get config from Core
  let state = Initializer::get()?;
  let config = state.config_manager().get_override_config(path)?;

  // Format using library formatter
  let formatted = library::formatter::format_text(
    &Article::new(content),
    &config.unwrap_or_default().fmt,
  );

  // Return full document replacement
  let line_count = content.lines().count() as u32;
  let last_line = content.lines().last().unwrap_or("");
  let last_char = last_line.chars().count() as u32;

  Ok(Some(vec![TextEdit {
    range: Range {
      start: Position { line: 0, character: 0 },
      end: Position { line: line_count.saturating_sub(1), character: last_char },
    },
    new_text: formatted.content_ref().to_string(),
  }]))
}
```

## Anti-Patterns

| Don't                                              | Do Instead                                                              |
| -------------------------------------------------- | ----------------------------------------------------------------------- |
| Use `println!` for logging in LSP handlers         | Use `Client::log_message()` for editor display, `eprintln!` for stderr  |
| Mix LSP protocol handling with business logic      | Keep `lsp/` module focused on protocol, delegate to Core for formatting |
| Use blocking operations in async handlers          | Use `tokio::sync::RwLock` for document storage, not `std::sync::Mutex`  |
| Call `Initializer::get()` without error handling   | Handle missing initialization gracefully with fallback defaults         |
| Return `Result` with generic error for LSP methods | Use `tower_lsp::jsonrpc::Result` which maps to JSON-RPC errors          |
| Hold write locks across await points               | Scope lock guards to minimize lock duration                             |
| Parse `Url` as file path manually                  | Use `url.to_file_path()` which handles platform differences             |

## When to Use

Load this skill when working on:

- `projects/cli/src/lsp/**/*.rs` - LSP-specific code
- Implementing new LSP capabilities (hover, completion, diagnostics)
- Document synchronization handlers (`did_open`, `did_change`, `did_close`)
- Formatting providers using Core library
- LSP server lifecycle management
- JSON-RPC communication with editors

**Differentiation from other skills:**

- Use `cli-dev` for: CLI commands, bridge management, runtime discovery
- Use `core-dev` for: Config management, state initialization, formatter logic
- Use `lsp-dev` for: LSP protocol implementation, document handlers, server capabilities (this skill)

**tower_lsp Resources:**

- Trait methods: `LanguageServer` trait in `tower_lsp::LanguageServer`
- Types: `lsp_types` crate for protocol types
- Error handling: `tower_lsp::jsonrpc::Result` for method returns
