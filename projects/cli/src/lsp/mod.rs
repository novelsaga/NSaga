mod backend;
mod completion;
mod position;

pub use backend::Backend;
#[allow(unused_imports)]
pub use completion::{build_completion_candidates, extract_active_prefix};
#[allow(unused_imports)]
pub use position::{offset_to_position, position_to_offset};
use tower_lsp::LspService;

/// 启动 LSP 服务器
pub async fn start() {
  eprintln!("NovelSaga LSP Server starting...");

  // Get stdin/stdout for LSP communication
  let stdin = tokio::io::stdin();
  let stdout = tokio::io::stdout();

  // Create the LSP service
  let (service, socket) = LspService::new(Backend::new);

  eprintln!("Starting LSP server...");

  // Run the server
  tower_lsp::Server::new(stdin, stdout, socket).serve(service).await;

  eprintln!("LSP server finished");

  // LSP exit notification received - exit cleanly
  eprintln!("NovelSaga LSP Server exiting...");
  std::process::exit(0);
}
