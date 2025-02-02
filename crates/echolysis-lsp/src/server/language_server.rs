use tower_lsp::{jsonrpc, lsp_types, LanguageServer};

use super::Server;

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(
        &self,
        params: lsp_types::InitializeParams,
    ) -> jsonrpc::Result<lsp_types::InitializeResult> {
        self.watch(&params.workspace_folders.unwrap_or_default())
            .await;

        Ok(lsp_types::InitializeResult {
            capabilities: lsp_types::ServerCapabilities {
                text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Kind(
                    lsp_types::TextDocumentSyncKind::FULL,
                )),
                workspace: Some(lsp_types::WorkspaceServerCapabilities {
                    workspace_folders: Some(lsp_types::WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(lsp_types::OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            server_info: Some(lsp_types::ServerInfo {
                name: "echolysis-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        self.stop().await;
        Ok(())
    }

    async fn did_change(&self, params: lsp_types::DidChangeTextDocumentParams) {
        if self.is_stopped() {
            return;
        }
        if let Ok(path) = params.text_document.uri.to_file_path() {
            self.on_insert(vec![(path, Some(&params.content_changes[0].text))])
                .await;
        }
    }

    async fn did_change_workspace_folders(
        &self,
        params: lsp_types::DidChangeWorkspaceFoldersParams,
    ) {
        if self.is_stopped() {
            return;
        }
        self.unwatch(&params.event.removed).await;
        self.watch(&params.event.added).await;
    }
}
