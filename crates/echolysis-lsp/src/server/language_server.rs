use std::sync::Arc;

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
        Ok(())
    }

    async fn did_open(&self, params: lsp_types::DidOpenTextDocumentParams) {
        self.on_insert(&[(
            params.text_document.uri,
            Some(Arc::new(params.text_document.text)),
        )])
        .await;
    }

    async fn did_close(&self, params: lsp_types::DidCloseTextDocumentParams) {
        self.clear_diagnostic(&[params.text_document.uri], None)
            .await;
    }

    async fn did_change(&self, mut params: lsp_types::DidChangeTextDocumentParams) {
        self.on_insert(&[(
            params.text_document.uri,
            Some(Arc::new(params.content_changes.swap_remove(0).text)),
        )])
        .await;
    }

    async fn did_change_workspace_folders(
        &self,
        params: lsp_types::DidChangeWorkspaceFoldersParams,
    ) {
        self.unwatch(&params.event.removed).await;
        self.watch(&params.event.added).await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> jsonrpc::Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Find a range that contains the clicked position
        let matching_locations = self
            .duplicate_locations
            .lock()
            .iter()
            .find(|locations| {
                locations.iter().any(|location| {
                    location.uri == uri
                        && location.range.start.line <= position.line
                        && position.line <= location.range.end.line
                        && (position.line != location.range.start.line
                            || position.character >= location.range.start.character)
                        && (position.line != location.range.end.line
                            || position.character <= location.range.end.character)
                })
            })
            .map(|locations| {
                locations
                    .iter()
                    .filter(|location| {
                        location.uri != uri
                            || (location.range.start.line != position.line
                                && location.range.end.line != position.line
                                && location.range.start.character != position.character
                                && location.range.end.character != position.character)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            });

        if let Some(locations) = matching_locations {
            if !locations.is_empty() {
                return Ok(Some(GotoDefinitionResponse::Array(locations.clone())));
            }
        }
        Ok(None)
    }
}
