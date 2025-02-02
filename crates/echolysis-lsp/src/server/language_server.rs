use tower_lsp::{
    jsonrpc,
    lsp_types::{self, GotoDefinitionParams, GotoDefinitionResponse},
    LanguageServer,
};

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
                text_document_sync: Some(lsp_types::TextDocumentSyncCapability::Options(
                    lsp_types::TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(lsp_types::TextDocumentSyncKind::FULL),
                        ..Default::default()
                    },
                )),
                definition_provider: Some(lsp_types::OneOf::Left(true)),
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

    async fn did_open(&self, params: lsp_types::DidOpenTextDocumentParams) {
        if self.is_stopped() {
            return;
        }
        if let Ok(path) = params.text_document.uri.to_file_path() {
            self.on_insert(vec![(path, Some(&params.text_document.text))])
                .await;
        }
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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        self.client
            .log_message(
                lsp_types::MessageType::ERROR,
                format!(
                    "Goto definition requested at uri: {:?}, position: {:?}",
                    uri, position
                ),
            )
            .await;

        // Log all stored locations for debugging
        self.client
            .log_message(
                lsp_types::MessageType::ERROR,
                format!(
                    "Total stored location entries: {}",
                    self.duplicate_locations.len()
                ),
            )
            .await;

        for entry in self.duplicate_locations.iter() {
            self.client
                .log_message(
                    lsp_types::MessageType::ERROR,
                    format!(
                        "Stored range: {:?} -> {} locations",
                        entry.key().range,
                        entry.value().len()
                    ),
                )
                .await;
        }

        // Find a range that contains the clicked position
        let matching_location = self.duplicate_locations.iter().find(|entry| {
            entry.key().uri == uri
                && entry.key().range.start.line <= position.line
                && position.line <= entry.key().range.end.line
                && (position.line != entry.key().range.start.line
                    || position.character >= entry.key().range.start.character)
                && (position.line != entry.key().range.end.line
                    || position.character <= entry.key().range.end.character)
        });

        if let Some(entry) = matching_location {
            let locations = entry.value();
            self.client
                .log_message(
                    lsp_types::MessageType::ERROR,
                    format!("Found {} locations", locations.len()),
                )
                .await;

            if !locations.is_empty() {
                return Ok(Some(GotoDefinitionResponse::Array(locations.clone())));
            }
        } else {
            self.client
                .log_message(
                    lsp_types::MessageType::ERROR,
                    "No locations found".to_string(),
                )
                .await;
        }

        Ok(None)
    }
}
